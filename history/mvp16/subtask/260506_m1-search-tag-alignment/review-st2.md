# ST-2 리뷰 결과 (step-5)

> 일시: 2026-05-06 | 리뷰어: Claude + Codex
> 대상: ST-2 — `tag_search_keyword()` 정밀화 + 한국어 키워드 추가

---

## 합의 (진행 가능)

| # | 항목 |
|---|------|
| 1 | 전체 12개 태그 한꺼번에 재작성 — 범위 설정 타당 |
| 2 | `mobile`, `software`, `tech` 단독 사용 제거 — 방향 맞음 |
| 3 | 한국어 기사 노출 목표 — 타당 |
| 4 | T-01~T-04 테스트 방향 — 필요 (단, 보강 필요) |
| 5 | E-01 `other` 분기 유지 — 맞음 |

---

## 수정 필요

### 1. 한국어 키워드 — ST-2 범위에서 제거

**근거**:
- 현재 코드 주석 자체가 "Tavily는 영어 쿼리에 최적화, 한/영 혼합 시 관련성 저하"라고 명시 (`feed.rs:30`)
- 최종 쿼리 형태: `"{search_keyword} latest news{suffix}"` — 한국어 추가 시 `"iOS app development Android ... 모바일 앱 개발 latest news GPT transformer"` 같은 긴 혼합 쿼리가 됨. 관련도 저하 가능성 큼.
- 한국어 노출은 ST-3의 `country:"kr"` 파라미터나 KR 멀티패스로 해결하는 것이 일관됨.

**결정**: ST-2는 영어 키워드 정밀화까지만. 한국어 키워드는 ST-3 실측 결과 이후 합류 여부 결정.

### 2. 키워드 설계 — 일부 제안이 여전히 느슨함

- `"모바일 개발" => "... mobile SDK ..."` — `SDK`는 광고 SDK, 게임 SDK도 잡을 수 있음. `iOS app development`, `Android app development` 축으로 더 좁힐 것.
- `"오픈소스" => "open source project GitHub developer library"` — `project`, `library`는 범용적. `GitHub`, `repository`, `maintainer`, `OSS` 등 소프트웨어 문맥이 강한 단어 우선.
- **권장 원칙**: 정밀화 = "금지어 없는 태그별 필수어 집합". `game`, `gaming`은 모바일 개발에서 금지. `pharmaceutical`, `R&D`는 오픈소스에서 금지.

### 3. 테스트 보강 필요

현재 T-01~T-04에 추가해야 할 테스트:

| # | 추가 테스트 케이스 |
|---|------------------|
| T-05 | `other` 분기: 매핑에 없는 태그명이 그대로 반환되는지 검증 |
| T-06 | 개인화 suffix 조립 확인: `tag_search_keyword()` 출력 + `" latest news{suffix}"` 형태가 유지되는지 |
| T-07 | 기존 `get_feed` 통합 테스트 전체 통과 확인 (`cargo test` 전체 — keyword 변경 시 mock query 깨질 수 있음) |
| T-08 | 문제 태그 금지어 미포함 검증: "모바일 개발"에 `"game"`, "오픈소스"에 `"pharmaceutical"` 미포함 |

### 4. 리스크: 쿼리 강화 → 빈 결과 → 캐시/안정성 영향

- 현재 체인: `Ok(_)` 빈 결과 시 warn 로그 + 다음 엔진 시도. 전 엔진 0건 시 `AppError::Internal` (`search_chain.rs:53`)
- 빈 결과 → `search_failed_count` 증가 → `FEED_CACHE_TTL_PARTIAL (1분)` 적용. 즉 키워드 정밀화가 캐시 TTL과 호출량에 직접 영향.
- **대응**: E-02 "검색 결과 수 급감 없음"은 단위 테스트 불가. ST-8 E2E 항목으로 이동 + "12개 태그 탭 0건 발생 없음" 확인.

### 5. `other` 분기 운영 정책 미결

- 새 태그 추가 시 raw tag명이 쿼리로 나감. 운영 로그에서 감지하는 장치 없음.
- **권장**: `other` 분기에 `tracing::warn!(tag_name = other, "tag_search_keyword: unknown tag, using raw name")` 추가. (이번 ST-2에서 처리)

---

## 합의된 구현 범위 (step-6 진입 기준)

| 항목 | 범위 |
|------|------|
| 키워드 재작성 | 전체 12개, 영어 전용, 테크 도메인 특화 |
| 한국어 키워드 | ST-2 제외. ST-3 결과 후 판단 |
| `other` 분기 warn 로그 | 추가 |
| 테스트 | T-01~T-08 (보강) |
| 비용 영향 | ST-7에서 검토 (Tavily 호출 수 = 기존 동일, ST-2 단계에서 증가 없음) |

---

## 수정 필요 → step-4 재진입 없이 step-6에서 반영

모든 수정은 step-6 구현에서 직접 반영. step-4 재진입 불필요.
