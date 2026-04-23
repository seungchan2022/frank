# 검색 엔진 파이프라인 심층 분석

**날짜**: 2026-04-23  
**유형**: architecture  
**대상**: 검색 엔진 파이프라인 전체 구조  
**목적**: BUG-004 수정 방안 문서화용 팩트 기반 분석

---

## 1. 검색 엔진 종류 (3개)

| 엔진 | 어댑터 파일 | `source_name()` 반환값 | SearchPort 구현 |
|------|------------|----------------------|----------------|
| Tavily | `server/src/infra/tavily.rs` | `"tavily"` | O |
| Exa | `server/src/infra/exa.rs` | `"exa"` | O |
| Firecrawl | `server/src/infra/firecrawl.rs` | `"firecrawl"` | O (CrawlPort도 겸함) |

---

## 2. 폴백 체인 구성

### 조립 위치: `server/src/main.rs:42-46`

```rust
let search_chain: Arc<dyn SearchChainPort> = Arc::new(SearchFallbackChain::new(vec![
    Box::new(TavilyAdapter::new(&config.tavily_api_key)),
    Box::new(ExaAdapter::new(&config.exa_api_key)),
    Box::new(FirecrawlAdapter::new(&config.firecrawl_api_key)),
]));
```

**폴백 순서**: Tavily → Exa → Firecrawl (인덱스 0 → 1 → 2)

### 폴백 로직: `server/src/infra/search_chain.rs`

`SearchFallbackChain::search()` 동작:
1. `sources` 벡터를 순서대로 순회
2. 각 엔진에서 `search()` 호출
3. 결과가 비어 있지 않으면 **즉시 `(Vec<SearchResult>, source_name)`을 반환하고 중단**
4. 결과가 비었거나 에러면 다음 엔진으로 넘어감
5. 전부 실패 시 마지막 에러(또는 "All search sources returned empty results") 반환

**핵심**: 폴백은 **태그 단위가 아닌 쿼리 단위**. 태그 A가 Tavily에서 성공하면 태그 B도 같은 체인으로 독립 실행됨.

---

## 3. 각 엔진의 API 파라미터 (팩트 기반)

### 3-1. Tavily (`/search` POST)

```json
{
  "query": "<검색어>",
  "max_results": <usize>,
  "search_depth": "advanced",
  "include_answer": false,
  "time_range": "week"
}
```

- **인증**: `Authorization: Bearer <API_KEY>` 헤더
- **base_url**: `https://api.tavily.com`
- **엔드포인트**: `POST /search`
- **`topic` 파라미터**: **없음** ← BUG-004 원인
- **`time_range: "week"`**: 있음 (주간 필터 적용 중)
- **응답 필드**: `results[].{title, url, content, published_date}`
- **og:image**: 검색 결과 URL 각각에 대해 `fetch_og_image()` 병렬 크롤링 (join_all)
- **retry**: `RetryConfig::for_search()` — 최대 3회, 100ms 지수 백오프, 2MB 제한
- **retryable 상태코드**: 429, 500, 502, 503, 504

### 3-2. Exa (`/search` POST)

```json
{
  "query": "<검색어>",
  "numResults": <usize>,
  "contents": {
    "highlights": {
      "numSentences": 3,
      "highlightsPerUrl": 1
    }
  }
}
```

- **인증**: `x-api-key: <API_KEY>` 헤더
- **base_url**: `https://api.exa.ai`
- **엔드포인트**: `POST /search`
- **`type` 파라미터**: **없음** ← BUG-004 수정 대상
- **`startPublishedDate` 파라미터**: **없음** ← BUG-004 수정 대상
- **응답 필드**: `results[].{title, url, highlights[], publishedDate}`
  - `title`: `Option<String>` (null이면 빈 문자열로 폴백)
  - `highlights`: `Option<Vec<String>>` (첫 번째 하이라이트만 snippet으로 사용)
- **og:image**: Tavily와 동일하게 병렬 크롤링
- **retry**: `RetryConfig::for_search()` — Tavily와 동일
- **snippet 정제**: `clean_snippet()` 함수 적용 (HTML 태그 제거, 300자 문장 경계 절단)

### 3-3. Firecrawl (`/v1/search` POST)

```json
{
  "query": "<검색어>",
  "limit": <usize>
}
```

- **인증**: `Authorization: Bearer <API_KEY>` 헤더
- **base_url**: `https://api.firecrawl.dev`
- **엔드포인트**: `POST /v1/search`
- **응답 필드**: `data[].{title, url, description}`
  - `url`: None이면 필터링 (filter_map)
- **og:image**: **크롤링 없음** (코드 주석: "Firecrawl API는 image_url 미제공")
- **published_at**: **없음** (항상 None)
- **retry**: `RetryConfig::for_search()`

---

## 4. 수집 서비스 레이어 — 실제 호출 위치

```
GET /api/me/feed (HTTP 엔드포인트)
  └── api/feed.rs: get_feed<D>()
        ├── db.get_user_tags()          — 사용자 구독 태그 조회
        ├── db.list_tags()              — 태그 이름 맵 생성
        ├── feed_cache.get()            — 캐시 조회 (HIT이면 즉시 반환)
        ├── db.get_like_count()         — 개인화 활성화 여부 (>= 3)
        ├── db.get_top_keywords()       — 태그별 상위 키워드 (개인화 시)
        ├── [태그 수]개의 future 생성
        │     search_query = "{tag_name} latest news{keyword_suffix}"
        │     chain.search(search_query, 5)
        │       └── SearchFallbackChain::search()
        │             ├── TavilyAdapter::search()   — 1순위
        │             ├── ExaAdapter::search()      — 2순위 (Tavily 실패 시)
        │             └── FirecrawlAdapter::search() — 3순위 (Exa도 실패 시)
        ├── futures::future::join_all(futures)  — 모든 태그 병렬 실행
        ├── is_homepage_url() 필터링     — path 세그먼트 <= 1 제거
        ├── normalize_url() 기반 중복 제거
        └── feed_cache.set()            — 결과 캐시 저장 (TTL 5분)
```

---

## 5. 공통 인프라 (`http_utils.rs`)

모든 검색 어댑터가 공유하는 유틸리티:

| 함수/상수 | 역할 |
|-----------|------|
| `send_with_retry()` | RequestBuilder 팩토리 기반 재시도, 지수 백오프 |
| `read_body_limited()` | chunked 응답도 커버하는 사이즈 제한 본문 읽기 |
| `fetch_og_image()` | 기사 URL 크롤링 → og:image 추출 (5초 타임아웃) |
| `extract_og_image()` | HTML에서 `<meta property="og:image">` 파싱 |
| `OG_IMAGE_TIMEOUT_SECS = 5` | og:image 크롤링 타임아웃 |
| `OG_IMAGE_READ_LIMIT = 64KB` | og:image용 HTML 최대 읽기 크기 |

---

## 6. BUG-004 수정을 위한 구체적 변경 방안

### 증상 재확인

- Tavily가 뉴스 카테고리/태그 인덱스 페이지를 반환
- 예: `"Data science recent news | AI Business"`, `"data science News & Articles - IEEE Spectrum"`
- 원인: 일반 웹 검색 모드 → listing 페이지 포함됨

### 방안 A: API 파라미터 수정 (권장, 최우선)

**Tavily** (`tavily.rs:67-74`):
```rust
let body = serde_json::json!({
    "query": query,
    "max_results": max_results,
    "search_depth": "advanced",
    "include_answer": false,
    "time_range": "week",
    "topic": "news",          // ← 추가: 뉴스 전용 검색 모드
});
```
주의: Tavily `topic: "news"` + `time_range: "week"` 조합 시 동작 검증 필요.  
Tavily 공식 문서 기준 `topic`과 `time_range` 동시 사용 가능 여부 확인 필수.

**Exa** (`exa.rs:109-119`):
```rust
let now = chrono::Utc::now();
let week_ago = now - chrono::Duration::days(7);
let body = serde_json::json!({
    "query": query,
    "numResults": max_results,
    "type": "news",                                     // ← 추가
    "startPublishedDate": week_ago.to_rfc3339(),        // ← 추가
    "contents": {
        "highlights": {
            "numSentences": 3,
            "highlightsPerUrl": 1
        }
    }
});
```

**Firecrawl**: 추가 파라미터 확인 후 적용 여부 결정. 현재 `published_at` 미제공이므로 3순위 폴백에서 의존도 낮음.

### 방안 B: URL 패턴 후처리 필터 (보완책, 방안 A와 병행)

`feed.rs`의 `is_homepage_url()` 함수가 이미 존재하나 path 세그먼트 1개 이하만 차단.  
listing 패턴 확장 필터를 추가:

```rust
// feed.rs에 추가할 listing 패턴 필터
fn is_listing_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    let listing_patterns = [
        "/tag/", "/tags/", "/category/", "/categories/",
        "/topic/", "/topics/", "/section/", "/archive/",
        "/news/", "/articles/",  // 단, path가 이것으로 끝나는 경우만
    ];
    // path 마지막 세그먼트가 listing 패턴인지 확인
    listing_patterns.iter().any(|p| lower.contains(p) && {
        let after = lower.split(p).last().unwrap_or("");
        // 패턴 이후 의미 있는 슬러그가 없으면 listing
        after.trim_end_matches('/').is_empty() || !after.contains('/')
    })
}
```

주의: 이 패턴은 `/news/some-article` 같은 정상 기사 URL도 차단할 수 있어 신중하게 적용.

### 방안 C: 현행 `is_homepage_url()` 강화 (최소 변경)

현재 로직: path 세그먼트 개수 <= 1이면 차단.  
강화 방향: `listing_keywords`를 마지막 세그먼트와 비교하는 방식 추가.

---

## 7. 의존 환경변수 목록

| 변수명 | 필수 | 용도 |
|--------|------|------|
| `TAVILY_API_KEY` | 필수 | TavilyAdapter 인증 |
| `EXA_API_KEY` | 필수 | ExaAdapter 인증 |
| `FIRECRAWL_API_KEY` | 필수 | FirecrawlAdapter 인증 (검색+크롤 겸용) |

---

## 8. 테스트 커버리지 현황

| 파일 | 핵심 테스트 케이스 |
|------|-------------------|
| `search_chain.rs` | 폴백 성공, 전체 실패, 빈 결과 스킵, 마지막 에러 반환, 빈 소스 에러, trait 경유, debug 포맷 (총 8개) |
| `tavily.rs` | retry, 사이즈 제한, 비2xx, JSON 파싱, 네트워크 실패, og:image 성공/실패, time_range 파라미터 검증 (총 8개) |
| `exa.rs` | retry, 사이즈 제한, 비2xx, JSON 파싱, 네트워크 실패, title null, og:image 성공/실패, clean_snippet 6종 (총 14개) |
| `firecrawl.rs` | search/scrape 각 6개 (총 12개) |
| `http_utils.rs` | retry, 백오프, 사이즈 제한, og:image 파싱 (총 16개) |

BUG-004 수정 시 추가해야 할 테스트:
- Tavily: `topic: "news"` 파라미터 포함 여부 검증 (`body_partial_json` matcher)
- Exa: `type: "news"` + `startPublishedDate` 포함 여부 검증
- feed.rs: listing URL (e.g. `/tag/ai`) 필터링 검증
