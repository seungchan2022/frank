# M2 서브태스크 목록: 텍스트·LLM 출력 정제

> MVP: 16 | 마일스톤: M2 | 생성: 2026-05-06
> 메인태스크: 피드 snippet/본문 sanitize + Groq 한국어 출력 제약 + occupation 무관 범용 insight 복원 (서버)

## step-1 미결정 사항 처리 결과

| 항목 | 결정 | 근거 |
|------|------|------|
| B1 sanitize 정책 | `clean_snippet` 인라인 마크다운 제거 (plain text 변환) | 피드 카드에 마크다운 렌더러 없음. `**bold**` → `bold` plain text 변환이 가장 단순하고 안전. |
| C1 후처리 정책 | 프롬프트 제약 강화 우선 + 후처리 없음 | 한자 감지 재요청은 호출 빈도 2배. 비용 정책(`project_api_cost_policy`) 위반. 프롬프트 제약(`한국어로만 작성`, `한자/중국어 금지`)이 선행. |
| C3 기존 favorites.insight 백필 | 보존 (신규만 범용 적용) | 기존 데이터 재생성은 불필요한 LLM 호출. 기존 데이터는 그대로, 새 요약부터 범용 insight. |

---

## 서브태스크 목록

> **수정 대상 집중**: ST-1과 ST-2는 동일 파일(`exa.rs`) 수정. ST-3는 동일 파일(`groq.rs`) 수정. ST-2와 ST-3는 독립적.

| # | ID | 내용 | 유형 | 의존 | 예상 |
|---|-----|------|------|------|------|
| 1 | ST-1 | `clean_snippet` 인라인 마크다운 제거 + 단위 테스트 (T-01~T-03) | feature | — | 1h |
| 2 | ST-2 | Tavily/Exa `clean_snippet` 호출 범위 확인 (다른 기사 raw 혼입 차단) | research | ST-1 | 0.5h |
| 3 | ST-3 | Groq 프롬프트 한국어 전용 제약 강화 + `occupation=None` 시 insight 범용 생성 | feature | — | 1.5h |
| 4 | ST-4 | `fake_llm` 보강 + 통합 테스트 (T-04~T-07) | test | ST-1, ST-3 | 1h |
| 5 | ST-5 | E2E 시나리오 3종 본인 직접 확인 | chore | ST-4 | 0.5h |

## 의존성 DAG

```
ST-1 (clean_snippet 마크다운 제거)
  └── ST-2 (혼입 차단 범위 확인)
             └── ST-4 (fake_llm + 통합테스트) ── ST-5 (E2E)

ST-3 (Groq 프롬프트 강화) ──────────────────────┘
```

ST-1, ST-3는 독립 시작 가능.
ST-2는 ST-1 완료 후 clean_snippet 범위 확인.
ST-4는 ST-1, ST-3 완료 후.
ST-5는 ST-4 통과 후.

---

## ST-1: `clean_snippet` 인라인 마크다운 제거

**파일**: `server/src/infra/exa.rs` → `clean_snippet()` 함수

**현재 상태**: HTML 태그, 마크다운 헤더(`#`), 플레이스홀더(`[...]`)는 제거하지만 인라인 마크다운은 미처리.

**추가할 변환**:
- `**text**` → `text` (굵기)
- `*text*` → `text` (이탤릭)
- `_text_` → `text` (이탤릭 대안)
- `[text](url)` → `text` (링크)

**구현 위치**: `clean_snippet()` 기존 파이프라인 1단계(HTML 제거) 직후에 삽입 또는 별도 단계로 추가.

**엣지 케이스**:
- `**text` (닫힘 없음) → 텍스트 그대로 유지 (R-02: 정규식 non-greedy 패턴)
- `[서울=뉴스핌]` 같은 기존 인라인 출처 패턴 — 이미 `process_snippet_line`에서 처리. 충돌 없음.
- `[...]` 플레이스홀더 — 기존 step 3에서 처리. 마크다운 링크 패턴과 다름 (`(url)` 부분 없음).

**단위 테스트 (T-01~T-03)**:
```rust
// T-01: **bold** 제거
assert_eq!(clean_snippet("**굵은** 텍스트"), "굵은 텍스트");
// T-02: [text](url) → text
assert_eq!(clean_snippet("[링크](https://example.com) 본문"), "링크 본문");
// T-03: 복합 패턴 (HTML + 마크다운)
let input = "<p>**title**</p> [link](url) 내용 *이탤릭*";
let result = clean_snippet(input);
assert!(!result.contains("**"), "bold 제거");
assert!(!result.contains("[link]"), "링크 텍스트 제거 — url 제거");
assert!(result.contains("title"), "텍스트 보존");
assert!(result.contains("link"), "링크 텍스트 보존");
assert!(result.contains("내용"), "일반 본문 보존");
```

---

## ST-2: 다른 기사 raw 혼입 차단 범위 확인

**조사 대상**:
1. `tavily.rs` — `clean_snippet()` 호출 위치 (`use crate::infra::exa::clean_snippet` 이미 import됨)
2. `exa.rs` — `clean_snippet()` 호출 위치
3. `feed.rs` — snippet 필드가 `SearchResult`에서 `FeedItem`으로 전달되는 경로

**판단 기준**: ST-1에서 인라인 마크다운 제거가 추가되면 Tavily/Exa 모두 동일 함수를 통과하므로 별도 처리 불필요할 가능성 높음.

**실제 혼입 원인 확인**: M2_text_llm_output.md의 "다른 기사의 raw markdown까지 snippet에 섞여 들어옴" 케이스 — Tavily `content` 필드에 여러 기사 요약이 연결되어 반환되는 경우. `clean_snippet`으로 마크다운 제거 후 텍스트 혼입 여전한지 확인.

---

## ST-3: Groq 프롬프트 한국어 제약 + 범용 insight 복원

**파일**: `server/src/infra/groq.rs`

### C1 — 한국어 전용 제약 강화

**수정 대상**: `SYSTEM_PROMPT_NO_OCCUPATION`, `SYSTEM_PROMPT_WITH_OCCUPATION`

**추가할 제약 문구** (system prompt 말미):
```
CRITICAL: Write ONLY in Korean (한국어). Do NOT use Chinese characters (漢字/한자), Japanese characters, or any non-Korean script. All output must be in pure Korean (hangul + standard punctuation only).
```

**적용 위치**: 두 프롬프트 상수 모두. `SYSTEM_PROMPT_REWRITE`는 이미 "Be in Korean" 명시 → 동일 문구 추가.

### C3 — `occupation=None`일 때 범용 insight 생성

**현재 코드** (`groq.rs:393–403`):
```rust
let (system_prompt, user_message) = if let Some(ref occ) = occupation {
    (SYSTEM_PROMPT_WITH_OCCUPATION, ...)
} else {
    (SYSTEM_PROMPT_NO_OCCUPATION, ...)  // insight 없음
};
```

**수정 방향**: `SYSTEM_PROMPT_NO_OCCUPATION`에 `insight` 필드 추가 (범용 2-3문장 분석).

**변경 후 흐름**:
- `occupation=None` → `SYSTEM_PROMPT_NO_OCCUPATION` (insight 범용 포함) → insight non-null
- `occupation=Some` → `SYSTEM_PROMPT_WITH_OCCUPATION` (insight 직업 맞춤) → insight non-null

**insight 파싱 코드 수정** (`groq.rs:454–469`):
```rust
// 현재: occupation.is_some() 일 때만 insight 파싱
// 수정: 항상 insight 파싱 (None 아님)
let insight = Some(
    parsed["insight"]
        .as_str()
        .ok_or_else(|| AppError::Internal("LLM response missing 'insight' field".to_string()))?
        .to_string(),
);
```

**`summarize_with_occupation` 서비스 주석 업데이트** (`summary_service.rs:74–78`):
```rust
// 현재: "occupation=None: 기존 summarize와 동일 동작 (insight null)"
// 수정: "occupation=None: 범용 insight 포함, occupation=Some: 직업 맞춤 insight"
```

---

## ST-4: fake_llm 보강 + 통합 테스트

**파일**: `server/src/infra/fake_llm.rs`

**현재 fake_llm**: insight 반환 여부 확인 필요.

**보강 항목**:
- `FakeLlmAdapter`의 `summarize_with_occupation(occupation=None)` → insight non-null 반환 확인
- `FakeLlmAdapter`의 `summarize_with_occupation(occupation=Some)` → insight non-null 반환 확인

**통합 테스트 (T-04~T-07)**:
- T-04: `fake_llm` — occupation Some/None 모두 insight non-null 반환
- T-05: occupation 없는 `summarize_with_occupation` → `SummarizeResponse.insight` non-null
- T-06: occupation 있는 `summarize_with_occupation` → insight non-null (회귀)
- T-07: `cargo test` 전체 통과

---

## ST-5: E2E 시나리오 3종 (본인 직접 확인)

| # | 시나리오 | 합격 기준 |
|---|---------|-----------|
| E-01 | 피드/디테일 raw 마크다운 노출 0건 | snippet에 `**`, `[text](url)`, `_text_` 없음 |
| E-02 | 한국어 기사 요약 → 한자 0건 | 요약/insight에 한자(`查看` 등) 없음 |
| E-03 | 직업 미설정 상태 요약하기 → insight 노출 | `SummarizeResponse.insight` non-null |

---

## 파일 경로 힌트

- `clean_snippet` 구현: `server/src/infra/exa.rs:43` (함수 시작)
- Tavily clean_snippet 호출: `server/src/infra/tavily.rs:8` (import), 검색 결과 매핑부
- Groq 프롬프트 상수: `server/src/infra/groq.rs:17–29`
- `summarize_with_occupation` 분기: `server/src/infra/groq.rs:382–489`
- 서비스 주석: `server/src/services/summary_service.rs:74–78`
- fake_llm: `server/src/infra/fake_llm.rs`

## 비용 영향 (KPI Hard 게이트: $0 유지)

| 변경 | 호출 증가 | 판정 |
|------|----------|------|
| C1 프롬프트 제약 강화 | 0 (재요청 없음) | 안전 |
| C3 insight 범용 복원 | 0 (동일 호출) | 안전 |
| B1 sanitize | 0 (서버 내 처리) | 안전 |

한자 재요청 전략 기각 → Groq 호출 수 변화 없음.

## DoD (M2 전체)

- [x] ST-1: `clean_snippet` 인라인 마크다운 제거 + T-01~T-03 통과
- [x] ST-2: 혼입 차단 범위 확인 완료 (추가 수정 있으면 반영)
- [x] ST-3: Groq 프롬프트 한국어 제약 추가 + occupation=None insight 범용 생성
- [x] ST-4: fake_llm 보강 + T-04~T-07 통과 + `cargo test` 전체 통과
- [x] ST-5: E2E 3종 본인 직접 통과
