# ST-2: A1-a + A2-a 키워드 매핑 정밀화 + 한국어 키워드 추가

> 유형: feature | 의존: 없음 | 병렬: ST-1과 독립, ST-3의 선행 작업 | 예상: 1.5h
> **통합**: 구 ST-4(한국어 키워드 추가)를 이 서브태스크에 합산. 동일 함수 수정이므로 별도 실행 불필요.

## 목적

`tag_search_keyword()` 매핑에서 느슨한 키워드를 제거하고 태그별 핵심 키워드만 남겨 무관 기사 혼입을 줄인다.

## 파일

`server/src/api/feed.rs` → `tag_search_keyword()`

## 현재 상태

```rust
"모바일 개발" => "mobile development iOS Android Swift Kotlin",
"오픈소스"   => "open source software developer tools",
```

**문제**: "mobile development"는 포켓몬 게임(모바일 게임) 기사를 끌어들인다. "open source software developer tools"는 의약품 R&D 도구 기사를 끌어들일 수 있다.

## 인터뷰 결정 (step-4)

- **수정 범위**: 전체 12개 태그 매핑 재작성 (문제 태그만이 아님)
- **근거**: "모바일 개발", "오픈소스"는 사용자가 제시한 예시일 뿐, 나머지 태그도 동일한 문제를 가질 수 있음

## 작업

1. 전체 12개 태그 키워드를 테크 도메인에 특화된 정밀 키워드로 재작성
2. 과도하게 넓은 단어(`mobile`, `software`, `tech` 등 단독 사용) 제거
3. **한국어 키워드 병행 추가** (구 ST-4 통합): 각 태그에 한국어 키워드 추가 → Tavily KR 기사 유도
4. 단위 테스트 갱신 — 새 키워드 매핑 검증

### 한국어 키워드 추가 방식 (구 ST-4 방안 A)

```rust
"모바일 개발" => "iOS app development Android Swift Kotlin mobile SDK 모바일 앱 개발",
"AI/ML"       => "artificial intelligence machine learning LLM 인공지능 머신러닝",
// ... 나머지 태그도 동일 패턴
```

Tavily 단일 호출에서 KR 기사 노출 여부는 ST-3 구현 후 E2E-2로 검증. KR 기사가 나오지 않으면 ST-3에서 `country:"kr"` 파라미터로 보완.

## 변경 예시

| 태그 | 현재 | 제안 |
|------|------|------|
| 모바일 개발 | mobile development iOS Android Swift Kotlin | iOS app development Android Swift Kotlin mobile SDK |
| 오픈소스 | open source software developer tools | open source project GitHub developer library |
| 스타트업 | startup tech entrepreneurship product launch | tech startup funding product hunt YC launch |

## 완료 기준

- [x] `tag_search_keyword()` 모든 매핑 검토 + 정밀화 (영어 전용, 리뷰 결정 반영)
- [x] `cargo test tag_search_keyword` 통과 (6개 테스트 OK)
- [x] 변경 전/후 키워드 비교 주석 추가

> **한국어 키워드 추가 (F-05)**: ST-3로 이연. 리뷰에서 한/영 혼합 쿼리 관련성 저하 우려로 제외 결정.

## Feature List
<!-- size: 중형 | count: 18 | skip: false -->

### 기능
- [x] F-01 tag_search_keyword() 전체 12개 태그 키워드 재작성 (테크 도메인 특화)
- [x] F-02 "모바일 개발" — mobile game 계열 키워드 배제, iOS/Android 앱 개발 특화
- [x] F-03 "오픈소스" — R&D/의약품 계열 배제, GitHub/라이브러리/OSS 특화
- [x] F-04 나머지 9개 태그 키워드 각각 느슨한 단어 제거 + 테크 특화
- [-] F-05 전체 12개 태그에 한국어 키워드 병행 추가 → ST-3로 이연 (리뷰 결정: Tavily 한/영 혼합 쿼리 관련성 저하 우려)

### 엣지
- [x] E-01 태그명이 매핑에 없는 경우(other) — 태그명 그대로 반환 동작 유지 확인 (T-05)
- [~] E-02 키워드 재작성 후 검색 결과 수 급감 없음 확인 → ST-8 E2E로 이동 (리뷰 결정: 단위 테스트 불가, 실측 필요)

### 에러
- [x] R-01 새 키워드로 Tavily 검색 시 빈 결과 반환 케이스 처리 확인 (기존 search_chain 로직 유지)

### 테스트
- [x] T-01 tag_search_keyword() 단위 테스트 — 전체 12개 태그 매핑 검증 (영어 전용)
- [x] T-02 기존 get_feed 통합 테스트 전체 통과 (cargo test: 404 passed)
- [x] T-03 "모바일 개발" 키워드에 "game" 계열 단어 미포함 검증 테스트
- [-] T-04 각 태그 키워드에 한국어 단어 ≥1개 포함 검증 → N/A (F-05 ST-3 이연으로 불필요)
- [x] T-05 `other` 분기: 매핑에 없는 태그명이 그대로 반환되는지 검증
- [x] T-06 suffix 조립 확인: `tag_search_keyword()` 출력 + `" latest news"` 형태 유지
- [x] T-07 기존 `get_feed` 통합 테스트 전체 통과 (`cargo test` 전체 — 404 passed)
- [x] T-08 문제 태그 금지어 미포함: "오픈소스"에 `"pharmaceutical"` 미포함

### 회귀
- [x] G-01 SearchPort 인터페이스 변경 없음 확인 (Exa/Firecrawl 영향 없음)
- [x] G-02 FeedItemResponse.tag_id: Option<Uuid> 응답 스키마 변경 없음 확인
- [x] G-03 make_cache_key() 동작 변경 없음 확인
