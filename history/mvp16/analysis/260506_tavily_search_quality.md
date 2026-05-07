# Tavily 검색 품질 심층 분석

**날짜**: 2026-05-06  
**유형**: 아키텍처·전략 분석  
**대상**: `server/src/infra/tavily.rs`, `server/src/api/feed.rs`

---

## TL;DR

무관 기사 혼입과 한국어 기사 0건은 **서로 다른 레이어의 다른 문제**다. 이 둘을 하나의 "키워드 튜닝" 문제로 묶어서 접근했기 때문에 세 차례 시도가 모두 수렴하지 못했다.

- 무관 기사 혼입 → **Tavily 인덱스·랭킹 특성 + 도메인 필터 미적용** 문제 (쿼리 레이어)
- 한국어 기사 0건 → **Tavily 인덱스 자체가 영어 뉴스 최적화** (인덱스 레이어, 쿼리로 해결 불가)

---

## 1. 문제 분리

### 1-A. 무관 기사 혼입 (NFL, 광업 기사)

**근본 원인 3가지:**

| # | 원인 | 증거 |
|---|------|------|
| R1 | `topic: "news"` 인덱스가 스포츠·정치·일반 뉴스 전체 포함 | Tavily docs: "politics, **sports**, and major current events" |
| R2 | 쿼리에 추가된 한국어 키워드("오픈소스", "웹 개발")가 영어 랭커에서 가중치 분산 유발 | 영어 랭커는 한글 토큰 처리 불안정, NFL 같은 고빈도 영어 뉴스가 스코어 역전 |
| R3 | `score` 필드를 수신하고 있지 않아 후처리 필터링 불가 | `TavilyResult` struct에 `score` 없음, 저빈도 관련 기사와 고빈도 무관 기사 구분 수단 없음 |

**키워드 추가가 효과 없는 이유:**  
Tavily는 키워드 매칭 검색이 아니라 벡터 기반 관련성 랭킹을 사용한다. "오픈소스 GitHub repository OSS" 같은 긴 키워드 문자열은 더 관련 있는 기사를 끌어올리는 게 아니라, 랭킹 가중치를 여러 토큰에 분산시켜 오히려 고빈도 무관 기사(NFL = 전 세계 뉴스 트래픽 최상위)에 역전될 여지를 만든다.

### 1-B. 한국어 기사 0건

**근본 원인: Tavily API 한계 (파라미터로 해결 불가)**

- **`country` 파라미터**: `topic: "news"` 와 함께 사용 불가 (docs 명시: "Available only if topic is `general`"). 현재 코드에서 이미 제거됨.
- **`language` 파라미터**: Tavily에 존재하지 않음. API 스펙에 없음.
- **쿼리에 한국어 키워드 추가**: 영어 랭커에 한글 토큰을 섞는 것. Tavily 인덱스 자체가 영어 뉴스 집계 사이트(TechCrunch, Wired, Ars Technica 등) 중심이므로, 한국어 키워드가 있어도 한국어 기사가 인덱싱되어 있지 않으면 반환 불가.

**결론: 한국어 기사는 Tavily 파라미터 튜닝으로는 원천적으로 해결 불가.**

---

## 2. 현재 코드에서 즉시 활용 가능한 레버 (미사용 상태)

### 2-A. `score` 필드 (무료, 즉시 적용 가능)

Tavily 응답에 `score` float이 포함되어 있으나 현재 `TavilyResult` struct에 없어 버려진다.

```rust
// 현재 (score 누락)
struct TavilyResult {
    title: String,
    url: String,
    content: Option<String>,
    published_date: Option<String>,
}

// 수정 후
struct TavilyResult {
    title: String,
    url: String,
    content: Option<String>,
    published_date: Option<String>,
    score: Option<f64>,  // 추가
}
```

`score`를 `SearchResult`에 전파하고, `feed.rs`에서 예컨대 `score < 0.3` 기사를 필터링하면 NFL·광업 기사 같은 저관련 결과를 제거할 수 있다.

### 2-B. `include_domains` 화이트리스트 (기술 뉴스 도메인 한정)

Tavily docs: max 300개 도메인 지정 가능. `topic: "news"`와 함께 작동 확인됨.

```json
"include_domains": [
  "techcrunch.com",
  "theverge.com",
  "arstechnica.com",
  "wired.com",
  "thenewstack.io",
  "infoq.com",
  "dev.to",
  "github.blog",
  "engineering.fb.com",
  "blog.cloudflare.com"
]
```

이것이 무관 기사 혼입 차단에 **가장 직접적인 레버**다. 기술 뉴스 전문 도메인만 허용하면 NFL 기사가 섞일 방법이 없다.

단점: 인덱싱 범위가 좁아져 결과 수 감소 가능. `max_results=20`을 유지하면서 `include_domains`를 30~50개로 세팅하면 균형점을 찾을 수 있다.

---

## 3. 개선안 A/B/C

### 안 A: `score` 후처리 필터 + 쿼리 정리 (최소 변경, 빠른 적용)

**변경 범위**: `tavily.rs` (score 역직렬화), `feed.rs` 또는 `tavily.rs` (score 필터), `feed.rs`의 한국어 키워드 혼합 제거.

**구체 작업**:
1. `TavilyResult`에 `score: Option<f64>` 추가, `SearchResult`에도 전파
2. Tavily 어댑터 내부에서 `score < 0.4` 결과 필터링 (임계값은 진단 데이터로 보정)
3. `tag_search_keyword()`에서 한국어 키워드 suffix 제거 (영어 쿼리 순수화)
4. 한국어 기사 목표 → 포기 또는 별도 소스로 분리 (이 안에서는 포기)

**장점**: 구현 간단, 테스트 용이, 기존 아키텍처 변경 없음  
**단점**: score 임계값이 임의적 (0.4가 최선인지 모름, 진단 데이터 필요). 광업·스포츠 기사를 완전 차단 보장 안 됨.

**적합 시점**: 지금 당장 MVP 내 빠른 개선이 목표일 때.

---

### 안 B: `include_domains` 기술 도메인 화이트리스트 (권장)

**변경 범위**: `tavily.rs`의 요청 바디, 도메인 리스트 관리 파일 추가.

**구체 작업**:
1. `tavily.rs` 요청 바디에 `include_domains` 추가 (기술 뉴스 전문 도메인 30~50개)
2. 도메인 리스트를 config 주입 가능한 `Vec<String>`으로 분리 (테스트 가능성 확보)
3. 한국어 기사를 위한 한국어 기술 도메인 추가 (`zdnet.co.kr`, `bloter.net`, `itworld.co.kr`, `etnews.com`)
4. 한국어 키워드 suffix는 유지 (한국어 도메인 기사 내 관련성 확보 목적)

**장점**: 무관 기사 혼입을 근본적으로 차단. 한국어 기사도 일부 노출 가능 (한국 IT 전문지 포함 시). 도메인 리스트는 이해하기 쉽고 조정 용이.  
**단점**: 도메인 리스트 관리 부담. 새로운 기술 블로그가 리스트에 없으면 누락. 결과 수 감소 가능성.

**적합 시점**: 품질이 최우선이고, 도메인 큐레이션을 수동으로 관리할 의향이 있을 때. **이 안이 현실적으로 가장 효과적이다.**

한국어 기사 포함 시 검증 필요 도메인 예시:
- `zdnet.co.kr` — ZDNet 코리아 (IT 전문)
- `bloter.net` — 블로터 (IT 스타트업)
- `itworld.co.kr` — IT World 코리아
- `etnews.com` — 전자신문
- `geek-news.mtx.kr` — GeekNews (개발자 커뮤니티 큐레이션)

---

### 안 C: 한국어 기사 전용 소스 추가 (장기 해결)

**전제**: 한국어 기사 노출이 hard requirement일 경우.

**변경 범위**: 새 어댑터 추가, `SearchFallbackChain` 수정, Naver News API 또는 RSS 연동.

**구체 작업**:
1. `NaverNewsAdapter` (Naver 뉴스 검색 API, 무료 한도 25,000/일) 또는 `RssAdapter` (GeekNews/요즘IT RSS) 구현
2. `SearchFallbackChain`을 "폴백 체인"에서 "병렬 멀티소스 머지"로 개선
3. 언어 기반 라우팅: 한국어 키워드 포함 태그는 Naver, 나머지는 Tavily
4. 결과 머지 후 중복 제거 (현재 `normalize_url` 재사용 가능)

**장점**: 한국어 기사 근본 해결. Naver News API는 무료이고 한국 IT 기사 커버리지 우수.  
**단점**: 구현 복잡도 높음 (새 어댑터 + 머지 로직 + 인증). MVP 내에서 완료하기 어려울 수 있음. Naver 개발자 앱 등록 필요.

**적합 시점**: 한국어 기사가 사용자 핵심 가치이고, 별도 마일스톤을 배정할 수 있을 때.

---

## 4. 권장 액션 플랜

| 우선순위 | 작업 | 안 | 소요 |
|----------|------|---|------|
| 1 | `TavilyResult`에 `score` 추가, score < 0.4 필터 적용 | A | 2시간 |
| 2 | `tag_search_keyword()`에서 한국어 suffix 제거 (영어 쿼리 순수화) | A | 30분 |
| 3 | `include_domains` 기술 도메인 화이트리스트 30개 추가 | B | 2시간 |
| 4 | 한국어 IT 도메인 5개 포함 (zdnet.co.kr 등) + E2E 검증 | B | 1시간 |
| 5 | (다음 MVP) Naver News 어댑터 추가 | C | 1~2일 |

1~4번은 현재 MVP 마일스톤 내 적용 가능하다. 5번은 별도 계획이 필요하다.

---

## 5. 한국어 기사 포기 여부 판단 기준

- **포기해도 됨**: 앱의 주 타겟이 영어 기사 기반 학습이고, 한국어 기사는 "있으면 좋음" 수준
- **포기하면 안 됨**: 한국 개발자 사용자가 한국어 기사로 공부하는 것이 핵심 유스케이스

현재 MEMORY에서 앱 동기는 "기술 트렌드 놓치지 않기 + 관심사 기반 파생 발견"이다. 영어 기사가 기술 트렌드 정보 밀도가 높으므로, **단기적으로는 안 A+B(영어 품질 개선)가 우선이고, 한국어는 안 C로 다음 MVP에서 처리**하는 것이 현실적이다.

---

## 참고: 검증된 Tavily API 사실 (2026-05-06 기준)

- `score` 필드: 응답에 포함됨 (0~1 float, 예: 0.81025416). 현재 미역직렬화.
- `include_domains`: 최대 300개, `topic: "news"`와 함께 작동.
- `country`: `topic: "general"`에서만 작동. `topic: "news"`와 함께 사용 불가.
- `language` 파라미터: 존재하지 않음.
- `topic: "news"`: 정치, 스포츠, 주요 현사 모두 포함. 기술 뉴스 한정 아님.
