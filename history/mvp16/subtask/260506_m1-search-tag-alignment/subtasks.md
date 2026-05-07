# M1 서브태스크 목록: 피드 검색·태그 정합

> MVP: 16 | 마일스톤: M1 | 생성: 2026-05-06
> 메인태스크: 피드 검색 결과의 태그 정합성 회복 — 무관 토픽 차단 + 한국어 기사 노출 + 태그-기사 매칭 일관화 (서버)

## A3 운영 정의 결정 (step-3에서 확정)

| 후보 | 내용 | 결정 |
|------|------|------|
| (i) 통계 안정 | 각 태그 검색을 정확하게 → 매칭 자연히 정합. 모델 변경 없음 | **선택** |
| (ii) N태그 carry | `FeedItem.tag_id` → `tags: Vec<Uuid>` 모델 변경 + M4 응답 스키마 영향 | 기각 |

**근거**: 피드는 ephemeral("훑고 넘김"). A3 진짜 문제(오픈소스 탭에 의약품 기사)는 검색 쿼리 부정확이 원인. 모델 변경 없이 쿼리 정밀화로 해결 가능. `FeedItemResponse.tag_id: Option<Uuid>` 응답 필드 그대로 유지 → M4 영향 없음.

**A3 확정 따른 ST-1/ST-6 범위 조정**: 사후 분류(케이스 C) 분기는 제거. ST-1은 "쿼리 정밀화 충분 여부" 진단으로 축소. ST-6는 케이스 A(스킵) 또는 케이스 B(통합 캐시)만 대상.

---

## 서브태스크 목록

> **병렬 실행 주의**: ST-2와 ST-4는 동일 함수(`tag_search_keyword()`) 수정, ST-3과 ST-5는 동일 파일(`tavily.rs`) 수정. 순차 실행 필수.

| # | ID | 내용 | 유형 | 의존 | 병렬 가능 | 예상 |
|---|-----|------|------|------|-----------|------|
| 1 | ST-1 | A3 진단: 쿼리 정밀화로 충분한지 확인 (통합 캐시 필요 여부) | research | — | — | 0.5h |
| 2 | ST-2 | A1-a + A2-a: `tag_search_keyword()` 키워드 매핑 정밀화 + 한국어 키워드 동시 추가 | feature | — | ST-3과 독립 | 1.5h |
| 3 | ST-3 | A1-b + A2-b: Tavily 파라미터 강화 — include_domains 화이트리스트 + KR country 파라미터 | feature | ST-2 | — | 2h |
| 4 | ST-6 | A3: 통합 캐시 전략 구현 (ST-1 결과 필요 시만 — 케이스 B) | feature | ST-1 | — | 1.5h |
| 5 | ST-7 | 단위·통합 테스트 + 비용 영향 검토 (`cargo test` + cost_log.md) | chore | ST-2, ST-3, ST-6 | — | 1h |
| 6 | ST-8 | E2E 시나리오 통과 본인 직접 확인 3건 | chore | ST-7 | — | 0.5h |

> **구 ST-4, ST-5 통합**: ST-4는 ST-2에, ST-5는 ST-3에 각각 합산. 별도 번호 제거.

## 의존성 DAG

```
ST-1 (A3 진단)
  └── ST-6 (A3 구현 — 케이스 B만 해당)
          └── ST-7

ST-2 (키워드+한국어 매핑)
  └── ST-3 (Tavily 파라미터 강화) ──┐
                                    ├── ST-7 ── ST-8
ST-6 ──────────────────────────────┘
```

ST-1, ST-2는 독립 시작 가능.
ST-3는 ST-2 완료 후 착수 (`tag_search_keyword()` 한국어 추가 결과 보고 Tavily KR 옵션 결정).
ST-6는 ST-1 결과 후 착수.

## 응답 스키마 계약 (M4 보호)

ST-2~ST-6 구현 후에도 `FeedItemResponse`의 `tag_id: Option<Uuid>` 필드는 변경하지 않는다.
M4 iOS 카드 태그 표시가 이 필드에 의존하므로 스키마 변경은 별도 M3 합의 없이 불가.

## 파일 경로 힌트

- 키워드 매핑: `server/src/api/feed.rs` → `tag_search_keyword()`
- Tavily 파라미터: `server/src/infra/tavily.rs` → `search()` body JSON
- SearchPort 계약: `server/src/domain/ports.rs` → `SearchPort::search()`
- 검색 체인: `server/src/infra/search_chain.rs`
- 비용 로그: `progress/mvp16/cost_log.md` (신규 생성)

## 비용 사전 계산 (KPI Hard 게이트: $0 유지)

| 항목 | 현재 | 변경 후 | 한도 | 판정 |
|------|------|---------|------|------|
| Tavily 호출/피드 요청 | 태그 수 N | ST-3이 `country:"kr"` 단일 파라미터면 동일 N | 1,000/월 | KR 멀티패스(옵션 2) 선택 시 2N → 한도 절반 이하면 허용 |
| Firecrawl og:image | 기사 수 K | 변경 없음 | 무료 | 변화 없음 |
| Exa | 폴백 시만 | 변경 없음 | 무료 | 변화 없음 |

> ST-3에서 KR 멀티패스(옵션 2) 선택 시 `progress/mvp16/cost_log.md`에 "월 예상 호출 수 = 2 × 태그 수 × 일일 요청 수 × 30" 계산 기록 필수. 한도 50% 초과 시 옵션 2 기각.

## DoD (M1 전체)

- [x] ST-2: `tag_search_keyword()` 전체 12개 태그 정밀화 + 한국어 키워드 추가 + 단위 테스트
- [x] ST-3: Tavily `country` 파라미터 제거 (스포츠/광업 혼입 부작용 확인) + 테스트 / `include_domains` → DEBT-MVP16-04 이관
- [-] N/A ST-6: A3 진단 결과 케이스 A (쿼리 정밀화로 충분) — 통합 캐시 구현 불필요
- [x] ST-7: `cargo test` 410 passed + `cost_log.md` 갱신 완료
- [x] ST-8: 본인 E2E — "모바일 개발" 탭 무관 기사 0건 ✅
- [~] deferred (DEBT-MVP16-05) ST-8: 본인 E2E — 임의 태그 탭 한국어 기사 ≥1건 → 0건 (Tavily 구조적 한계)
- [x] ST-8: 본인 E2E — 특정 태그 탭의 기사가 해당 태그 기사임 확인 (오분류 0건) ✅
- [x] ST-8: 배포 후 캐시 플러시 1회 강제 갱신 ✅
- [x] ST-8: 전체 12개 태그 탭 순회 — 0건 탭 없음 ✅

> **Known-limitation (A3)**: 현재 URL 중복 제거 구조에서 동일 기사의 tag_id 귀속은 `join_all` 도착 순으로 결정됨. "같은 기사가 두 탭에서 서로 다른 tag_id로 나타나는지" 검증은 현재 아키텍처에서 불가. 이는 의도된 동작이며 A3 문제("오픈소스 탭에 의약품 기사")의 근본 원인은 쿼리 정밀화(ST-2/ST-3)로 해결함.
