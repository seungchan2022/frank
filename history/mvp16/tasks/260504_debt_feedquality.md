# 태스크: 부채 해소 + 피드 품질 개선

> 작성일: 2026-05-04  
> 성격: 버그 수정 / 리팩토링 / 테스트 — MVP 아님  
> 상태: planning

---

## 배경

MVP15 완료 후 심층 분석을 통해 확인된 사항:

1. `debts.md`의 OPEN 부채 7개(MVP15-01~07)가 모두 유효하게 미구현 상태
2. DEBT-08(E2E 자동화)은 실제로 이미 구현됨 → RESOLVED 처리 필요
3. 피드 품질 개선 후보 4개 중 2개(키워드 매핑, 중복 제거)는 이미 구현됨
4. 실제로 새로 구현해야 할 피드 기능: **날짜 필터 + 소스 다양성** 2개

핵심 원칙 (이번 태스크부터 적용):
- **기능 구현 + E2E 시나리오는 같은 워크플로우 안에서 완료** — step-8에서 E2E까지 통과해야 커밋

---

## 작업 목록

### A. 피드 품질 개선 (신규 기능)

| ID | 내용 | 대상 파일 | 규모 |
|----|------|---------|------|
| A-1 | 발행 날짜 필터 (7일 이상 기사 제외) | `server/src/api/feed.rs` | 소 (~20줄) |
| A-2 | 소스 다양성 제한 (동일 도메인 상한) | `server/src/api/feed.rs` | 중 (~50줄) |

### B. OPEN 부채 해소

| ID | 내용 | 대상 파일 | 규모 |
|----|------|---------|------|
| B-1 | retry_classifier 402 → `quota_exhausted` 분류 | `server/src/bin/diagnose/retry_classifier.rs` | 소 (~5줄) |
| B-2 | 진단 바이너리 실행 환경 문서화 + 스크립트 | `scripts/run-diagnose.sh` 신규 | 소 (~30줄) |
| B-3 | Tavily `effective_max` 동적 감지 | `server/src/infra/tavily.rs` | 중 (~40줄) |
| B-4 | Exa `reset_at` NULL semantics 구현 | `server/src/infra/postgres_counters.rs` | 소 (~10줄) |
| B-5 | snippet invalid JSON escape 제거 | `server/src/infra/exa.rs` (clean_snippet) | 소 (~5줄) |
| B-6 | `infra → services` 레이어 의존 위반 수정 | `server/src/infra/counted_search.rs` 구조 변경 | 대 (파일 이동) |
| B-7 | DEBT-08 RESOLVED 처리 | `progress/debts.md` | 문서만 |
| ~~B-8~~ | ~~MVP15-05 spawn_blocking~~ | — | **이미 완료** |

### C. E2E 시나리오 추가

| ID | 내용 | 대상 | 규모 |
|----|------|------|------|
| C-1 | occupation → insight → rewrite → scrap (Playwright) | `web/e2e/` | 중 (~130줄) |
| C-2 | iOS rewrite 버튼 → 결과 표시 (XCUITest) | `ios/FrankUITests/` | 중 (~150줄) |

---

## 전체 규모

- 코드 변경: 약 400~500줄
- 파일: 8~10개 (신규 2개)
- 예상 소요: 워크플로우 1회

---

## 완료 조건

- [ ] A-1, A-2 피드 개선 + 단위 테스트 통과
- [ ] B-1~B-6 부채 전체 해소 + 단위 테스트 통과
- [ ] B-7 debts.md DEBT-08 RESOLVED 처리
- [ ] C-1, C-2 E2E 시나리오 작성 + 실제 실행 통과
- [ ] `cargo clippy`, `cargo test`, `npm run validate` 전체 통과

---

## 참고

- 부채 원문: `progress/debts.md`
- 피드 로직: `server/src/api/feed.rs`
- 진단 바이너리: `server/src/bin/diagnose/`
- 기존 E2E 패턴: `web/e2e/feed-like.spec.ts`, `ios/FrankUITests/FeedRefreshUITest.swift`
