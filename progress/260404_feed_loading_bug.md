# Feed 버그 수정

- 상태: 진행 전
- 브랜치: feature/fix-feed-loading
- 생성일: 2026-04-04

## 증상

피드 페이지(`/feed`)에서 **"Loading articles..."가 영구적으로 표시**되며 기사가 렌더링되지 않음.
네트워크 요청은 모두 200 OK로 성공하지만 UI에 반영되지 않는다.

### 스크린샷 관찰

- "My Feed" 헤더 + 태그 탭(전체, 모바일 개발) 표시
- "모바일 개발" 태그 선택 상태
- "Loading articles..." 메시지 무한 대기
- Refresh 버튼 "Loading..." 표시 (disabled)
- 콘솔 에러 1건

## 근본 원인 분석

### Bug 1: IntersectionObserver TypeError (핵심 원인)

**파일**: `web/src/routes/feed/+page.svelte:57-61`

```js
$effect(() => {
    if (sentinel) {
        observer.observe(sentinel);
        return () => observer.unobserve(sentinel!);  // BUG
    }
});
```

**문제**: cleanup 함수가 `sentinel`을 참조(reference)로 캡처함.
- `selectTag()` 호출 → `articles = []` → sentinel div가 DOM에서 제거 → `sentinel = null`
- $effect cleanup 실행 시점에 `sentinel`은 이미 null
- `observer.unobserve(null)` → **TypeError** 발생
- 에러가 `flush_queued_effects` → `update_effect` → `execute_effect_teardown` 콜스택에서 발생
- **Svelte 5 reactivity flush가 중단**되어 이후 상태 업데이트(articles, loading)가 DOM에 반영되지 않음

**콘솔 에러**:
```
TypeError: Failed to execute 'unobserve' on 'IntersectionObserver': parameter 1 is not of type 'Element'.
```

**수정 방향**: cleanup에서 sentinel 값을 로컬 변수로 캡처

```js
$effect(() => {
    const el = sentinel;
    if (el) {
        observer.observe(el);
        return () => observer.unobserve(el);  // el은 항상 Element
    }
});
```

### Bug 2: loadInitial() 다중 호출 (경합 조건)

**파일**: `web/src/routes/feed/+page.svelte:41-45`

```js
$effect(() => {
    if (auth.isAuthenticated) {
        loadInitial();  // 동기 가드 없이 호출
    }
});
```

**문제**: `auth.isAuthenticated`는 `!!session`이지만, `$effect`는 boolean 값이 아닌 **session 참조 자체**를 추적.
- 1회: `initAuth()` → `getSession()` → session 할당 → $effect
- 2회: `onAuthStateChange` `INITIAL_SESSION` → session 재할당(새 객체) → $effect
- 3회: `onAuthStateChange` `TOKEN_REFRESHED` → session 재할당 → $effect
- 참조가 바뀔 때마다 $effect 재실행 → `loadInitial()` 3회 호출
- 네트워크에서 확인: **articles 3회 + tags 3회** = 총 6건의 불필요한 중복 요청

**수정 방향**: `onMount`에서 1회만 호출하거나, `untrack`으로 session 추적 차단

### Bug 3: usedTags가 태그 전환 시 사라짐 (UX 문제)

**파일**: `web/src/routes/feed/+page.svelte:31-33`

```js
const usedTags = $derived(
    tags.filter((tag) => articles.some((a) => a.tag_id === tag.id) || selectedTagId === tag.id)
);
```

**문제**: `selectTag()` 호출 시 `articles = []`로 초기화됨.
- articles가 비어있으므로 `articles.some(...)` 조건은 항상 false
- selectedTagId에 해당하는 태그만 표시 → 다른 태그 탭이 일시적으로 사라짐
- 데이터 로드 완료 후에야 다시 나타남

**수정 방향**: tags 목록은 articles와 독립적으로 유지하거나, 전체 tags를 항상 표시

### Bug 4: 기사 원문 URL이 개별 기사가 아닌 사이트/섹션 URL (데이터 품질)

**파일**: `server/src/services/collect_service.rs` (수집 로직)

**문제**: 기사 수집(collect) 시 개별 기사 URL 대신 사이트 홈페이지 또는 뉴스 목록 페이지 URL이 저장됨.

**확인된 사례**:
| 기사 제목 | 저장된 URL | 문제 |
|-----------|-----------|------|
| Latest News: Business, Technology & Leadership - Fast Company | `https://www.fastcompany.com/news` | 뉴스 목록 페이지 |
| Latest AI & ML News, Insights, and Trends - Times of AI | `https://www.timesofai.com/` | 사이트 홈페이지 |

**영향**: "원문 보기" 링크가 실제 기사가 아닌 사이트 메인으로 이동하여 사용자가 원문을 찾을 수 없음.

**원인**: Tavily `search_depth: "basic"`에서 도메인 홈페이지 URL을 반환하는 것은 알려진 동작. 코드상 URL 필터링/검증 로직 없음 (tavily.rs:79에서 그대로 매핑).

**수정 방향**:
- `search_depth: "advanced"` 변경 또는 URL 패턴 필터링 (`/`만 있는 URL 제외)
- Firecrawl 본문 크롤링 추가 시, 크롤링 결과의 길이/품질 기반 필터링도 병행

### Bug 5: 기사 내용이 너무 빈약함 (데이터 품질)

**파일**: `server/src/infra/tavily.rs`, `server/src/services/collect_service.rs`

**문제**: 수집된 기사의 내용(snippet)이 1~2문장으로 매우 짧아 요약/인사이트 품질이 낮음.

**원인 체인**:
1. Tavily API에 `search_depth: "basic"` 사용 → 짧은 `content`만 반환
2. `SearchResult`에 `snippet`(짧은 텍스트)만 저장 → 기사 본문(full content) 미수집
3. LLM 요약이 이 빈약한 스니펫을 기반으로 생성 → 깊이 없는 요약/인사이트

**확인된 사례**:
- "Times of AI" 기사 스니펫: "Google has launched Gemini 3 and claims it to be the most intelligent model yet, with the best reasoning..." (1문장 잘림)
- 이 스니펫으로 생성된 요약도 1~2문장에 불과

**수정 방향**:
- Tavily는 검색(URL/제목/스니펫 수집)용으로 유지
- 수집 후 개별 기사 URL에 대해 **Firecrawl로 본문 크롤링** → `content` 필드에 저장
- Article 모델에 `content: Option<String>` 필드 추가
- LLM 요약은 snippet 대신 firecrawl로 가져온 **full content** 기반으로 생성
- LLM 요약 시 **한국어로 쉽게 풀어서** 제목/요약/인사이트 생성 (단순 번역 X, 이해하기 쉬운 표현)
- 원문 title은 별도 보존, 한국어 제목은 `title_ko` 등으로 저장
- 프롬프트 개선: 현재 `openrouter.rs`의 SYSTEM_PROMPT에 한국어 제목 + 쉬운 표현 지시 추가
- LLM 응답에 `title_ko` 필드 추가 → `LlmSummary` 모델 확장

### 모델 변경 + LLM 사용량 추적

**벤치마크 결과** (2026-04-04):
- `qwen/qwen3-235b-a22b`: JSON 출력 실패 (structured output 미호환), `finish_reason: length`
- `qwen/qwen3.5-plus-02-15`: JSON 정상 출력, thinking 정상 동작, 가격도 저렴

**모델 변경**: `qwen/qwen3-235b-a22b` → `qwen/qwen3.5-plus-02-15`

**수정 사항**:
- `config/mod.rs`: 기본 모델명 변경
- `openrouter.rs`: `response_format: { type: "json_object" }` 추가 (structured output)
- `openrouter.rs`: SYSTEM_PROMPT 개선 (한국어 쉬운 표현 + title_ko)
- `openrouter.rs`: `ChatResponse`에 `Usage` 구조체 추가 (현재 usage 파싱 안 함)
- DB 마이그레이션: `articles` 테이블에 `title_ko(text)`, `content(text)`, `llm_model(text)`, `prompt_tokens(int)`, `completion_tokens(int)` 추가 (모두 nullable)

**영향 범위** (심층분석 검증):
- `domain/models.rs`: Article + LlmSummary 확장
- `domain/ports.rs`: `update_article_summary` 시그니처 변경 + `CrawlPort` trait 신규
- `infra/openrouter.rs`: Usage 파싱 + title_ko 파싱 + response_format
- `infra/firecrawl.rs`: CrawlPort 구현 추가 (현재는 SearchPort만 구현)
- `infra/fake_db.rs`, `infra/fake_llm.rs`: 테스트용 어댑터 수정
- `infra/postgres_db.rs`: SELECT/UPDATE 쿼리 확장
- `services/summary_service.rs`: 토큰 정보 + title_ko 전달
- 테스트 코드: make_article 헬퍼, 어설션 수정
- 마이그레이션 SQL 1개 추가 (4번째)

## 서브태스크

| # | 태스크 | 파일 | 상태 |
|---|--------|------|------|
| 1 | IntersectionObserver cleanup에서 sentinel 로컬 캡처 | `+page.svelte` | 미완 |
| 2 | loadInitial 중복 호출 방지 (가드 or onMount) | `+page.svelte` | 미완 |
| 3 | usedTags → 전체 tags 표시로 변경 | `+page.svelte` | 미완 |
| 4 | collect_service에서 개별 기사 URL 올바르게 추출 | `collect_service.rs` | 미완 |
| 5 | CrawlPort trait 신규 정의 + Firecrawl scrape 구현 + Article에 content 추가 | `ports.rs` / `firecrawl.rs` / 모델 / DB | 미완 |
| 6 | LLM 모델 변경 (qwen3.5-plus) + 프롬프트 개선 + structured output | `openrouter.rs` / `config` | 미완 |
| 7 | LLM 사용량 추적: articles 테이블에 llm_model, prompt_tokens, completion_tokens 추가 | DB 마이그레이션 / 모델 | 미완 |
| 8 | 검증: 린트 + 타입체크 + 브라우저 확인 | — | 미완 |
