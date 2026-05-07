# ST-3: A1-b + A2-b Tavily 파라미터 강화 (무관 도메인 차단 + KR 노출)

> 유형: feature | **의존: ST-2** | 병렬: ST-1, ST-6과 독립 | 예상: 2h
> **통합**: 구 ST-5(KR 파라미터/멀티패스)를 이 서브태스크에 합산. 동일 파일(`tavily.rs`) 수정이므로 별도 실행 불필요.
> **ST-2 의존 이유**: ST-2의 한국어 키워드 추가 결과에 따라 `country:"kr"` 파라미터가 필요한지 결정. ST-2 완료 후 착수.

## 목적

Tavily API 호출 시 파라미터를 강화해:
1. 무관 도메인(게임, 의약품 등) 기사를 차단 (A1-b)
2. 한국어 기사가 ≥1건 노출되도록 보장 (A2-b, ST-2 결과 불충분 시)

## 파일

`server/src/infra/tavily.rs` → `search()` body JSON

## 현재 상태

```rust
let body = serde_json::json!({
    "query": query,
    "max_results": effective,
    "search_depth": "advanced",
    "include_answer": false,
    "time_range": "week",
    "topic": "news",
});
```

## 파트 1: 무관 도메인 차단 (A1-b)

### 작업 옵션 — 결정 기준 포함

| 옵션 | 방법 | 선택 조건 |
|------|------|-----------|
| **옵션 A (권장)** | `include_domains` 테크 화이트리스트 | ST-2 키워드 정밀화 후에도 게임/의약품 기사 혼입 지속 시. 한국어 도메인 반드시 포함 |
| 옵션 B | `exclude_domains` 명시 차단 | 혼입 도메인이 2~3개로 명확히 특정될 때만. whack-a-mole 위험 있어 1차 시도 후 A로 교체 가능 |

> **기각**: `exclude_domains` 단독 사용은 무한 확장 함정. ST-2로 근본 원인(느슨한 키워드)을 먼저 제거하고, 그래도 혼입되면 옵션 A로.

```json
// 옵션 A 예시 (한국어 도메인 포함 필수)
{
  "include_domains": [
    "techcrunch.com", "wired.com", "theverge.com", "arstechnica.com",
    "zdnet.co.kr", "bloter.net", "etnews.com", "itworld.co.kr"
  ]
}
```

## 파트 2: 한국어 기사 노출 (A2-b, 구 ST-5 통합)

### ST-2 결과에 따른 분기

#### ST-2 방안 A 충분 (KR 기사 나옴)
파트 2 구현 불필요 — 주석으로 "ST-2 한국어 키워드로 충분" 기록 후 완료.

#### ST-2 방안 A 불충분 (KR 기사 여전히 0건)
아래 중 하나 구현:

**옵션 1 (권장): `country:"kr"` 단일 파라미터**
```rust
let body = serde_json::json!({
    "query": query,
    "country": "kr",   // 추가
    "max_results": effective,
    "search_depth": "advanced",
    "topic": "news",
    "time_range": "week",
});
```
호출 수 변화 없음 — 비용 영향 없음.

**옵션 2: EN/KR 멀티패스 (호출 2배 — 최후 수단)**
```rust
let en_results = chain.search(&en_query, max/2).await;
let kr_results = chain.search(&kr_query, max/2).await;
// 합치기 + 중복 제거
```
선택 시 `progress/mvp16/cost_log.md`에 "월 예상 호출 수 = 2 × 태그 수 × 일일 요청 수 × 30" 계산 기록 필수.

## 폴백 경로 주의 (C1)

`include_domains` 필터는 `TavilyAdapter`에만 적용됨. Tavily가 0건 반환하거나 장애 시 `SearchFallbackChain`이 Exa → Firecrawl로 폴백하고 필터가 적용되지 않은 결과를 반환함. 이는 현재 아키텍처에서 **의도된 동작**으로 수용. 배포 시 "Tavily 장애 시 도메인 필터 효과 사라짐" 인지 하에 운영.

필터를 폴백 경로까지 강제해야 한다면 `SearchChainPort` 레이어에 정책 객체 주입 재설계 필요 (M1 범위 초과 → M3 이후 부채로 이관).

## 도메인 필터 적용 범위 제한 (M1)

`include_domains`를 전체 12개 태그에 전역 적용하면 `투자/VC`, `블록체인`, `보안` 등 출처 풀이 넓은 태그가 공동화될 위험 있음. 아래 규칙으로 적용:

- **필터 적용 대상**: "모바일 개발", "웹 개발", "AI/ML", "오픈소스" 4개 태그 (실제 오염 확인된 태그)
- **나머지 8개 태그**: 필터 없음 (ST-2 키워드 정밀화만 의존)
- 적용 후 해당 4개 태그 결과가 5건 미만이면 필터 없이 재시도 (안전망)

## 완료 기준

- [-] N/A (이후 이관) Tavily body에 도메인 필터 파라미터 추가 (4개 태그 선택 적용, 옵션 A 또는 B 선택 근거 주석) → DEBT-MVP16-04로 이관
- [x] KR 노출 방식 결정: `country` 파라미터 완전 제거 (E2E에서 스포츠/광업 혼입 부작용 확인)
- [x] `tavily_request_does_not_include_country` 테스트 추가 (country 미포함 검증)
- [-] N/A 멀티패스 미선택 — `cost_log.md` 비용 변화 없음 기록
- [x] 폴백 경로 미적용 사실 기존 주석으로 문서화

## 결정 요약 (2026-05-06)

**파트 1 (도메인 필터)**: 보류 → `DEBT-MVP16-04`
- `include_domains` 화이트리스트는 E2E 결과를 보고 이후 MVP에서 적용 예정
- 이번 M1에서는 `tag_search_keyword()` 키워드 정밀화(ST-2)만으로 A1 부분 해소

**파트 2 (KR 노출)**: `country` 파라미터 완전 제거
- `country:"kr"` → Tavily API 파라미터 오류 발생
- `country:"south korea"` → 한국 뉴스 집계 사이트 우선화 부작용 (스포츠/광업 기사 혼입)
- 한국어 기사 노출 실패 → `DEBT-MVP16-05` (Naver News API 어댑터) 이관
