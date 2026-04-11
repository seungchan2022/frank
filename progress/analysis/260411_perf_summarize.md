# 요약 기능 응답 속도 분석 보고서

**날짜**: 2026-04-11  
**대상**: `POST /me/summarize` 전체 파이프라인  
**목표**: 현재 병목 구조 파악 → 속도 개선 방안 A/B/C 제시

---

## 1. 현재 파이프라인 구조

```
[iOS]
  POST /me/summarize
       │
[server/src/services/summary_service.rs]
       │
       ├─ 1. SSRF 검증 (url_jail) ──────────── ~수ms
       │
       ├─ 2. Firecrawl.scrape() ───────────── 5~20s
       │   ├─ POST /v1/scrape (30s timeout)
       │   ├─ 2회 retry (base delay 100ms, exponential)
       │   └─ onlyMainContent: true, formats: ["markdown"]
       │
       ├─ 3. OpenRouter.summarize() ──────── 10~40s
       │   ├─ POST /api/v1/chat/completions (60s timeout)
       │   ├─ 1회 retry (base delay 200ms)
       │   ├─ 전체 markdown 텍스트 그대로 전달 (길이 제한 없음)
       │   └─ response_format: json_object (reasoning.exclude: true)
       │
       ├─ 4. favorites 업데이트 ──────────── ~수ms
       │
       └─ 5. JSON 응답 반환

[총 60s 오케스트레이션 타임아웃]
```

**실측 범위**: 최소 15s ~ 최대 60s (타임아웃 시 504)

---

## 2. 병목 원인 분석

### 2-1. 순차 처리 (피할 수 없는 구조적 한계)
Firecrawl이 콘텐츠를 가져와야 OpenRouter가 요약할 수 있다.  
두 호출은 의존 관계가 있어 병렬화 불가 — 총 지연 = 크롤 시간 + LLM 시간.

### 2-2. 스트리밍 없음 (체감 속도의 핵심 문제)
현재 구조:
- 서버: 전체 JSON 완성 후 한 번에 응답 반환 (`Json<SummarizeResponse>`)
- iOS: 응답 수신 전까지 로딩 스피너만 표시

OpenRouter는 SSE 스트리밍을 지원하지만 현재 `read_body_limited()`로 전체 대기 중.  
사용자는 첫 토큰이 생성된 후에도 마지막 토큰까지 기다려야 화면에 뭔가 보임.

### 2-3. 입력 길이 제한 없음 (LLM 추론 시간 직결)
Firecrawl `onlyMainContent: true`로 내비게이션·광고는 제거되지만,  
본문이 긴 기사(5,000~20,000자)는 전부 LLM에 전달됨.

`max_tokens: 800`으로 출력을 제한해도 **입력 토큰 수에 비례해 추론 시간이 늘어남**.  
요약에 필요한 정보는 앞 3,000~5,000자면 충분한 경우가 대부분.

### 2-4. LLM 모델 선택 (OpenRouter 경유 지연)
OpenRouter는 중간 라우팅 레이어 — 모델 선택에 따라:
- reasoning 모델(MiniMax M2.5 등): TTFT 3~10s, 전체 10~40s
- 빠른 모델(Llama 3.1 8B): TTFT 0.5~1s, 전체 3~8s

현재 코드 확인 시 모델명은 환경변수로 주입되어 런타임에 결정됨.

---

## 3. 현재 비용 구조

### 사용 중인 외부 API (요약 파이프라인)

| 서비스 | 용도 | 현재 플랜 추정 | 단가 |
|--------|------|--------------|------|
| **Firecrawl** | 기사 크롤 | Free 500 credits/month | Hobby $16/month (3,000 credits) |
| **OpenRouter** | LLM 요약 | Pay-as-you-go | 모델별 상이 |
| └ `qwen/qwen3.5-plus` | 현재 모델 | — | ~$0.50/1M input, ~$1.50/1M output |

### 요약 1회당 비용 추정

```
크롤: Firecrawl 1 credit
LLM:  입력 ~3,000 tokens + 출력 ~800 tokens
      = $0.0015 + $0.0012 = ~$0.003 / 1회

월 100회 사용 시: ~$0.30 + Firecrawl 100 credits
월 1,000회 사용 시: ~$3.00 + Firecrawl 1,000 credits (Hobby $16)
```

**현재 규모(개인 스터디앱)에서는 LLM 비용 자체는 크지 않다.**  
단, Firecrawl 크롤 비용은 요약 횟수에 비례해 증가.

---

## 4. 외부 리서치 요약 (빠른 LLM 옵션 + 비용)

| 제공자 | 모델 | TTFT | 전체 응답 | Input | Output | 무료 티어 |
|--------|------|------|----------|-------|--------|---------|
| **현재** | `qwen/qwen3.5-plus` via OpenRouter | ~3s | ~20-40s | ~$0.50/1M | ~$1.50/1M | 없음 |
| **Groq** | `llama-3.1-8b-instant` | ~80ms | ~2s | $0.05/1M | $0.08/1M | 30 RPM / 14,400 RPD |
| **Groq** | `llama-3.3-70b-versatile` | ~200ms | ~4s | $0.59/1M | $0.79/1M | 30 RPM / 14,400 RPD |
| **Google** | `gemini-2.5-flash` | ~500ms | ~5s | $0.075/1M | $0.30/1M | 15 RPM 무료 |
| **OpenRouter** | `qwen3-8b:free` | ~1s | ~8s | $0 | $0 | rate limit 있음 |

**비용 비교 (월 1,000회 요약 기준, 입력 3,000 + 출력 800 tokens):**

```
현재 (qwen3.5-plus):  $1.50 + $1.20 = ~$2.70/month
Groq llama-3.3-70b:   $0.18 + $0.06 = ~$0.24/month  ← 현재 대비 91% 절감
Groq llama-3.1-8b:    $0.015+ $0.006= ~$0.02/month  ← 거의 0원
Gemini 2.5 Flash:     $0.02 + $0.02 = ~$0.04/month
```

**결론**: Groq `llama-3.3-70b-versatile`은 현재 모델 대비 **속도 5~10배 빠르고 비용도 91% 절감**.  
무료 티어(14,400 RPD)는 개인 앱 규모에서 사실상 limitless.  
단, `response_format: json_object` 지원 여부 확인 필요 (미지원 시 프롬프트 엔지니어링으로 보완 가능).

---

## 5. 개선 방안

### A안 — LLM 입력 길이 제한 + 모델 교체 (즉시 적용 가능, 최고 효과)

**변경 내용:**
1. `summary_service.rs`에 content truncation 추가
2. OpenRouterAdapter의 모델을 Groq 호환 설정으로 교체 (또는 GroqAdapter 신규 작성)

```rust
// summary_service.rs — scrape 결과 전달 전 truncation
const MAX_CONTENT_CHARS: usize = 5_000;

let content = crawl.scrape(url).await?;
let content_trimmed = content.chars().take(MAX_CONTENT_CHARS).collect::<String>();

let response = llm.summarize(title, &content_trimmed).await?;
```

**예상 효과:**
- 속도: 전체 응답 20~40s → **3~8s** (5~10배 단축)
- 비용: 현재 ~$2.70/month(1,000회) → **~$0.24/month** (91% 절감)
- Groq 무료 티어 사용 시 LLM 비용 **0원** (14,400 RPD 이하)
- 구현 복잡도: **낮음** (truncation 1줄 + 환경변수 모델명 변경)
- 리스크: 긴 기사의 후반부 핵심 내용 누락 가능 (뉴스 기사 특성상 앞부분이 중요)

### B안 — SSE 스트리밍 구현 (중기, 체감 속도 극대화)

**변경 내용:**
1. 서버: `post_summarize` 핸들러를 `Sse<...>` 응답 타입으로 변경
2. OpenRouterAdapter: `stream: true` 파라미터 추가, `EventSource` 소비
3. iOS: `URLSession.bytes(for:)` async stream으로 부분 텍스트 수신 + 화면 실시간 업데이트

```rust
// 서버 응답 타입 변경 (개념)
pub async fn post_summarize_stream<D: DbPort>(
    ...
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // OpenRouter stream=true → 토큰 단위로 Event 방출
}
```

**예상 효과:**
- 첫 글자 화면 표시: 현재 15~40s → **1~3s**
- 체감 속도 40~90% 개선 (실제 처리 시간은 동일, **추가 비용 없음**)
- 구현 복잡도: **높음** (서버 응답 구조, iOS 파싱, 에러 처리 전면 변경)
- 리스크: 부분 JSON 처리 복잡 (현재 `response_format: json_object`와 충돌)

### C안 — iOS 로딩 UX 개선 + Firecrawl 대체 (즉시 적용, 최소 변경)

**변경 내용 1 — iOS UX:**
- 요약 버튼 탭 시 "분석 중... (최대 60초 소요)" 문구 + 진행 애니메이션
- 단계 표시: "기사 읽는 중..." → "요약 생성 중..." (서버 로직 변경 없이 타임라인 기반)

**변경 내용 2 — Firecrawl 대체:**
- Firecrawl scrape 대신 Jina Reader (`https://r.jina.ai/{url}`) 사용
- GET 요청 한 번으로 마크다운 반환, Firecrawl 대비 응답 속도 2~3배 빠름
- API 키 불필요 (무료), 단 rate limit 존재 (무료 RPM 제한)

```rust
// JinaAdapter (신규 — CrawlPort 구현)
impl CrawlPort for JinaAdapter {
    fn scrape(&self, url: &str) -> Pin<...> {
        let jina_url = format!("https://r.jina.ai/{url}");
        // GET 요청만으로 마크다운 반환
    }
}
```

**예상 효과:**
- 속도: Firecrawl 5~20s → Jina 1~5s (크롤 단계 단축)
- 비용: Firecrawl credits 소비 → **0원** (Jina 무료 플랜, 50 RPM)
- UX 개선으로 사용자 이탈 감소 (실제 속도가 아닌 체감 개선)
- 구현 복잡도: **낮음~중간**
- 리스크: Jina 무료 플랜 50 RPM 제한 (동시 요청 많으면 병목)

---

## 6. 권장 우선순위

```
즉시 (이번 스프린트):
  A안 - content truncation (1줄) + Groq 모델 교체
  → 속도 5~10배 + 비용 91% 절감, 코드 변경 최소

단기 (다음 마일스톤):
  C안 - Jina Reader 어댑터 교체
  → Firecrawl 크롤 비용 0원, 크롤 속도 추가 개선
  → A안 + C안 조합 시 요약 외부 API 비용 합계 ~$0/month (무료 tier 내)

중기 (MVPx):
  B안 - SSE 스트리밍
  → 추가 비용 없이 체감 속도 극대화, 아키텍처 변경 필요
```

**A안만 적용해도 20~40s → 3~8s + 비용 91% 절감.**  
**A안 + C안 조합하면 요약 기능 외부 API 비용이 사실상 0원.**

---

## 7. 관련 파일

| 파일 | 역할 |
|------|------|
| `server/src/services/summary_service.rs` | 파이프라인 오케스트레이션, truncation 추가 위치 |
| `server/src/infra/openrouter.rs` | LLM 어댑터 (모델 교체 또는 Groq 어댑터 신규) |
| `server/src/infra/firecrawl.rs` | 크롤 어댑터 (Jina로 교체 시 대상) |
| `server/src/domain/ports/mod.rs` | `CrawlPort` / `LlmPort` 트레이트 정의 |
| `ios/.../APISummarizeAdapter.swift` | iOS 요약 호출 (스트리밍 구현 시 변경 대상) |
