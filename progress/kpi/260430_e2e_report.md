# E2E 실행 리포트

> 날짜: 2026-04-30
> 실행자: /e2e 스킬 (M4 시나리오 완성)
> 브랜치: main
> 환경: local (build-for-testing 검증, 실 실행은 deploy.sh 선행 필요)

## 요약

| 플랫폼 | 전체 | 통과 | 실패 | 스킵 |
|--------|------|------|------|------|
| 웹 (Playwright) | 10 | - | - | - |
| iOS (XCUITest) | 6 | - | - | - |
| **합계** | **16** | **-** | **-** | **-** |

> 빌드 검증 (build-for-testing / svelte-check): 모두 PASS
> 실 E2E 실행 결과는 deploy.sh 선행 기동 후 별도 수집 필요

## 웹 E2E (Playwright) — 10개

| # | 시나리오 ID | 파일 | 테스트명 | 빌드 |
|---|------------|------|---------|------|
| 1 | smoke | e2e/smoke.spec.ts | Playwright 실행 환경 동작 확인 | ✅ |
| 2 | W-01 | e2e/feed-summary.spec.ts | DEBT-06: 스크랩 버튼 하단 고정 노출 | ✅ |
| 3 | W-01 | e2e/feed-summary.spec.ts | DEBT-07: 기사/요약 카드 시각적 구분 | ✅ |
| 4 | W-01 | e2e/feed-summary.spec.ts | 요약하기 버튼 탭 후 요약 결과 표시 | ✅ |
| 5 | W-01 | e2e/feed-summary.spec.ts | BUG-006: 요약 API 실패 시 에러+재시도 | ✅ |
| 6 | W-02 | e2e/tag-navigation.spec.ts | 태그 탭 클릭 시 피드 목록 전환 | ✅ |
| 7 | W-02 | e2e/tag-navigation.spec.ts | BUG-008: 탭 전환 시 컨테이너 재마운트 없음 | ✅ |
| 8 | W-03 | e2e/feed-like.spec.ts | DEBT-04: 좋아요 탭 시 URL 유지 | ✅ |
| 9 | W-03 | e2e/feed-like.spec.ts | DEBT-04: 좋아요 아이콘 ♡→♥ 토글 | ✅ |
| 10 | W-03 | e2e/feed-like.spec.ts | 좋아요/스크랩 독립 동작 | ✅ |

## iOS E2E (XCUITest) — 6개

| # | ID | 클래스 | 메서드 | 커버 항목 | 빌드 |
|---|----|--------|--------|-----------|------|
| 1 | I-01 | LoginFlowUITest | testEmailLoginToFeed | 로그인→피드+기사 로딩 확인 | ✅ |
| 2 | I-02 | CrossFeatureFlowUITest | testTagTabSwitching | BUG-008 탭전환 피드 유지 | ✅ |
| 3 | I-03 | M3UXImprovementsUITest | testDetailSummaryThenActionButton | BUG-006+DEBT-06 요약 후 버튼 | ✅ |
| 4 | I-04 | FeedRefreshUITest | testPullToRefreshUpdatesFeed | BUG-007 pull-to-refresh 2단계 Mock | ✅ |
| 5 | I-04 | FeedRefreshUITest | testPullToRefreshInMockMode | Mock 모드 RefreshControl 동작 | ✅ |
| 6 | I-05 | M3UXImprovementsUITest | testFeedLikeButtonDoesNotNavigateToDetail | DEBT-04 좋아요 단독 탭 | ✅ |

## 신규 생산 코드 변경

| 파일 | 변경 내용 |
|------|-----------|
| `MockArticleAdapter.swift` | `refreshSeed` 파라미터 + noCache 2단계 분기 |
| `MockFixtures.swift` | `refreshedFeedItems` fixture 추가 |
| `AppDependencies.swift` | `feed_refresh_2step` 시나리오 분기 |

## 블로커 수정 이력

| 블로커 | 원인 | 수정 |
|--------|------|------|
| W-02 MutationObserver 순서 버그 | `page.evaluate()`가 Promise 블록으로 클릭 이전에 observer disconnect | observer를 클릭 전 전역(`window.__obs__`) 설치, 클릭 후 결과 회수 + `expect(removedCount).toBe(0)` 추가 |
| I-04 retry 누락 | swipeDown 단발로는 RefreshControl 트리거 불안정 | `app.tables` 우선 접근 + 최대 2회 retry 로직 추가 |

## 환경 정보

- BASE_URL: http://localhost:5173
- iOS 시뮬레이터: iPhone 17 Pro
- 웹 빌드 검증: `svelte-check 0 ERRORS 0 WARNINGS`
- iOS 빌드 검증: `TEST BUILD SUCCEEDED`

## 비고

- 실 E2E 실행 전 `scripts/deploy.sh --target=api,front --native` 선행 필수
- W-02 BUG-008 어설션: grid 컨테이너 미존재 시(-1) 어설션 건너뜀 (태그 전환 후 빈 상태 정상 케이스)
- I-04: SwiftUI `List` → `UITableView` → `app.tables` 로 접근, collectionViews fallback 포함
