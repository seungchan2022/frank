# M2: 텍스트·LLM 출력 정제

> 프로젝트: Frank MVP16
> 상태: 완료
> 예상 기간: 2~3일
> 의존성: M1 (서버 우선 원칙. 검색 정합 회복 후 출력 정제)

## 목표

본문·요약·인사이트가 깨지지 않고 일관된 한국어로 출력되도록 텍스트 처리·LLM 프롬프트·노출 분기를 바로잡는다.

## 배경 (`_review_notes.md` §2 서버: 텍스트·LLM 출력)

실사용 피드백에서 다음 3건이 발견되었다 — B1(raw 마크다운 노출), C1(한국어 요약에 한자 혼입), C3(직업 미설정 시 인사이트 미노출).

## 포함 항목

### B1. snippet/본문에 raw 마크다운 노출

- **상황**: 피드 카드 snippet 또는 기사 본문 표시 영역
- **현재**: `***title***`, `[text](/path)`, `### 헤더` 같은 raw 마크다운이 그대로 출력됨. 어떤 기사는 다른 기사의 raw markdown까지 snippet에 섞여 들어옴 (예: dongascience 기사의 snippet에 KAIST/Perigee 기사 raw 포함).
- **기대**: 사용자에게는 plain text 또는 정상 렌더링
- **수정 결**: snippet/본문 처리 파이프라인에 sanitize 단계 추가. plain text 변환 / 마크다운 렌더링 일관 정책 결정 (step-1).

### C1. 한국어 요약에 한자 혼입

- **상황**: 한국어 기사 요약 결과 표시
- **현재**: Groq llama 모델이 한국어 출력에 일부 한자(`查看` 등) 섞음. 프롬프트가 출력 언어 제약 약함.
- **기대**: 한국어 출력에 한자/외국어 혼입 없음
- **수정 결**: 요약 프롬프트에 출력 언어 제약 명시 + 후처리에서 한자 감지 시 재요청 또는 변환. 모델 변경 검토 가능.

### C3. 직업 미설정 시 인사이트 미노출 (DEBT-MVP16-03)

- **상황**: 사용자가 설정에서 직업을 비워둔 상태로 기사 열기
- **현재**: 인사이트 자체가 노출 안 됨. 직업이 있어야만 인사이트 생성/노출.
- **기대**: 인사이트는 직업 무관 기본 노출. 직업이 있으면 직업 시각이 추가/대체. (요약·인사이트와 재작성은 독립 기능)
- **수정 결**: 인사이트 생성을 직업 의존 분기에서 분리. 기본 인사이트(범용) + 직업별 인사이트(있으면) 구조로.

## occupation 의미 매트릭스 (M2 ↔ M3 충돌 회피)

C3가 `profiles.occupation`을 인사이트 분기 입력으로 사용하는 한편, 같은 컬럼이 M3에서 D1(삭제 정합)·C2-bug(rewrite 다중 시점 키)로도 의미가 바뀐다. M2 done 시점에 M3 시작 전 매트릭스를 박제해 충돌을 차단한다.

| 차원 | 마일스톤 | occupation NULL일 때 동작 |
|------|---------|--------------------------|
| 인사이트 분기 | M2 C3 | 범용 인사이트 생성 (직업 무관) |
| 입력 정합 | M3 D1 | 명시적 NULL 저장 가능 |
| rewrite 저장 키 | M3 C2-bug | NULL 키 자체로 별도 시점 보존 OR rewrite 호출 차단 (M3 C4) |

**M2 done 직전**: 기존 favorites.insight 데이터(MVP15 시대까지 직업 의존으로 생성된 것)의 백필 정책을 step-1에서 결정. 후보 — (a) 그대로 보존하고 신규만 범용 적용, (b) 기존 데이터 일괄 폐기 후 재생성, (c) 사용자 occupation에 따라 라벨링.

## 미결정 (워크플로우에서 결정)

| 항목 | 결정 시점 |
|------|-----------|
| B1 sanitize 정책 (plain text 변환 vs 마크다운 정상 렌더링) | step-1 |
| C1 후처리 정책 (한자 감지 시 재요청 vs 변환 vs 모델 변경) | step-1 |
| C3 기존 favorites.insight 데이터 백필 정책 (보존 / 폐기 / 라벨링) | step-1 |

## 성공 기준 (Definition of Done)

- [ ] B1: snippet/본문 sanitize 파이프라인 적용 + 단위 테스트 (raw 마크다운 패턴 fixture)
- [ ] B1: 다른 기사 raw 콘텐츠가 snippet에 섞여 들어오는 케이스 차단
- [ ] C1: Groq 프롬프트에 한국어 출력 제약 명시 (system prompt 갱신)
- [ ] C1: 한자 감지 후처리 또는 재요청 로직 + 단위 테스트
- [ ] C3: occupation 무관 범용 insight 생성 (MVP9 시대 동작 복원) + occupation 있으면 직업 맞춤으로 업그레이드
- [ ] C3: `SummarizeResponse.insight` 필드가 occupation 미설정 시에도 non-null
- [ ] 본인 직접 사용: 한국어 기사 요약 → 한자 0건 확인 (E2E)
- [ ] 본인 직접 사용: 직업 미설정 상태에서 요약하기 → insight 노출 확인 (E2E)
- [ ] 본인 직접 사용: 피드/디테일 raw 마크다운 노출 0건 확인 (E2E)
- [ ] 서버 단위·통합 테스트 통과 (`cargo test`)
- [ ] 비용 영향: 한자 재요청 시 호출 빈도 측정 후 무료 한도 위반 0건 확인

## 워크플로우 진입점

```
/workflow "M2-text-llm-output"
```

**메인태스크**: 피드 snippet/본문 sanitize + Groq 한국어 출력 제약 + occupation 무관 범용 insight 복원 (서버).

## 아이템

| # | 아이템 | 유형 | 순서 | 상태 |
|---|--------|------|------|------|
| 1 | B1 sanitize 정책 결정 (plain vs 마크다운) | decision | 1 | 대기 |
| 2 | B1 sanitize 파이프라인 구현 + 단위 테스트 | feature | 2 | 대기 |
| 3 | C1 한국어 출력 제약 프롬프트 + 한자 후처리 | feature | 3 | 대기 |
| 4 | C3 범용 insight 복원 (`SYSTEM_PROMPT` 통합 또는 분기 제거) | feature | 4 | 대기 |
| 5 | E2E 시나리오 3종 통과 + 비용 영향 검토 | chore | 5 | 대기 |

## KPI (M2)

| 지표 | 측정 방법 | 목표 | 게이트 | 기준선 |
|---|---|---|---|---|
| 서버 테스트 통과 | `cargo test` | 전체 통과 | Hard | — |
| sanitize 단위 테스트 | `cargo test snippet` 또는 fixture 기반 테스트 | 통과 | Hard | — |
| raw 마크다운 노출 회귀 차단 | 본인 E2E: 피드/디테일 raw 마크다운 카운트 | 0건 | Hard | — |
| 한자 혼입 회귀 차단 | 본인 E2E: 한국어 요약에 한자 카운트 | 0건 | Hard | — |
| occupation 미설정 insight 노출 | 본인 E2E: 직업 빈 상태 요약하기 → `insight` 필드 | non-null | Hard | null |
| 무료 한도 위반 발생 0건 | `progress/mvp16/cost_log.md` 검토 | $0 유지 | Hard | — |

## 리스크

| 리스크 | 영향(H/M/L) | 대응 |
|--------|------------|------|
| sanitize 후 본문이 너무 짧아짐 → 사용성 저하 | M | 정책 결정(step-1) 시 마크다운 렌더링 옵션 검토. 길이 하한 보장 |
| 한자 후처리 재요청이 LLM 호출 빈도 2~3배 증가 | M | 변환(번역 또는 제거) 우선 검토. 재요청은 fallback. 비용 정책 메모리 정합 |
| C3 범용 insight 복원이 occupation insight 회귀 유발 | H | 기존 occupation prompt 보존 + 범용 prompt 분기로 통합. M3 C2-bug와 occupation 의미 변경 충돌 주의 (cross-milestone) |
| 다른 기사 raw가 snippet에 섞이는 원인이 검색 엔진 응답 자체 → M1과 의존 | M | M1 변경(검색 호출 파라미터)이 snippet 형태에 영향 가능. M1 완료 후 진입하여 응답 스키마 박제 |

## Feature List
<!-- size: 중형 | count: 20 | skip: false -->

### 기능
- [x] F-01 `clean_snippet`에 `**text**` → `text` 변환 추가
- [x] F-02 `clean_snippet`에 `*text*` → `text` 변환 추가
- [x] F-03 `clean_snippet`에 `[text](url)` → `text` 변환 추가 + `[](` unclosed 패턴 제거
- [x] F-04 `clean_snippet`에 `_text_` → `text` 변환 추가
- [x] F-05 `SYSTEM_PROMPT_SUMMARY`에 insight 필드 추가 (범용 2-3문장 분석)
- [x] F-06 `SYSTEM_PROMPT_SUMMARY`에 한국어 전용 제약 문구 추가
- [-] N/A (C3 occupation 분기 제거로 `SYSTEM_PROMPT_WITH_OCCUPATION` 상수 삭제됨) F-07 `SYSTEM_PROMPT_WITH_OCCUPATION`에 한국어 전용 제약 문구 추가
- [x] F-08 `summarize_with_occupation`에서 occupation 분기 제거 → 항상 범용 프롬프트 사용 + insight 파싱 분기 제거 → occupation 무관 항상 insight 파싱

### 엣지
- [-] E-01 N/A (이미 처리됨) — `exa.rs` `Option::map` 내부에서 `clean_snippet` 호출하므로 None 케이스 자동 안전
- [x] E-02 `**` 미닫힘 쌍(`**text` 끝 없음) — 텍스트 그대로 유지
- [x] E-03 occupation이 Some이어도 요약/인사이트 응답이 기존과 동일

### 에러
- [x] R-01 LLM 응답에 insight 필드 누락 시 AppError::Internal 반환
- [x] R-02 인라인 마크다운 제거 후 snippet이 빈 문자열 → None 처리

### 테스트
- [x] T-01 `clean_snippet` — `**bold**` 제거 단위 테스트 (fixture)
- [x] T-02 `clean_snippet` — `[text](url)` → `text` 단위 테스트
- [x] T-03 `clean_snippet` — 복합 패턴 (기존 HTML 제거 + 신규 마크다운 제거 혼합)
- [x] T-04 `fake_llm` — occupation Some/None 모두 insight non-null 반환 확인
- [x] T-05 occupation 없는 summarize 요청 → `SummarizeResponse.insight` non-null 통합 테스트
- [x] T-06 occupation 있는 summarize 요청 → insight 여전히 non-null (회귀)
- [x] T-07 `cargo test` 전체 통과 (438개)

## 참고

- 합의 노트: `progress/mvp16/_review_notes.md` §2.2 (서버: 텍스트·LLM 출력)
- 부채: `DEBT-MVP16-03` (`progress/debts.md`), `DEBT-MVP15-07` (snippet invalid escape)와 동일 파이프라인
- 관련 메모리: `project_api_cost_policy`
