# 알려진 버그 목록

발견된 버그를 기록. 다음 MVP 기능 구현 전에 수정 후 진행.

> 260428 상태 정리: BUG-001~004 모두 코드에서 수정 완료. 상세 분석: `progress/analysis/260428_full_debt-bugs.md`
> 260429 상태 정리: 실사용 테스트 중 BUG-006~010 신규 발견. MVP14에서 수정 예정.

---

## [BUG-001] 앱 첫 실행 시 태그/데이터 로딩 실패

**발견**: 2026-04-22 실기기 테스트 중
**상태**: ✅ **RESOLVED** (260428 코드 탐색 확인)
**재현**: 시뮬레이터·실기기 공통, 항상 재현됨
**증상**: 앱 시작 직후 "태그를 불러오지 못했습니다" 에러 표시 → 다시 시도 누르면 정상 동작
**원인**: Supabase 세션 복원이 완료되기 전에 API 요청이 먼저 나감 → Bearer 토큰 없음 → 401/연결 실패

콘솔 경고:
```
Initial session emitted after attempting to refresh the local stored session.
To opt-in to the new behavior now, set `emitLocalSessionAsInitialSession: true`
```

**수정 완료**: `AppDependencies.swift` — `SupabaseClientOptions(auth: .init(emitLocalSessionAsInitialSession: true))`
`FrankApp.swift` — `AuthState.checkingSession` 상태에서 `SplashView` 표시로 API 호출 차단

**우선순위**: 다음 MVP 시작 전 수정

---

## [BUG-002] 시뮬레이터/실기기 SERVER_URL 분리 안 됨

**발견**: 2026-04-23 시뮬레이터 테스트 중
**상태**: ✅ **RESOLVED** (260428 코드 탐색 확인)
**재현**: 실기기 테스트 후 시뮬레이터 전환 시 항상 재현
**증상**: `Config.xcconfig`의 `SERVER_URL`이 실기기용 LAN IP로 고정돼 있어 시뮬레이터에서 API 연결 실패 → 스플래시 화면 무한 대기

**원인**: `Config.xcconfig`에 `SERVER_URL = http://192.168.x.x:8080` 하드코딩.
시뮬레이터는 `localhost:8080` 접근 가능하지만 LAN IP로 연결 시도 → stall timeout

**수정 완료**: `ServerConfig.swift` — `#if targetEnvironment(simulator)` 분기로 시뮬레이터는 `localhost:8080` 컴파일 타임 고정. `Config.xcconfig`도 `localhost:8080` 기본값으로 업데이트.

**우선순위**: 다음 MVP 시작 전 수정

---

## [BUG-003] 즐겨찾기/오답 노트 태그 필터 없음

**발견**: 2026-04-23
**상태**: ✅ **RESOLVED** — MVP12에서 BUG-F로 흡수, 클라이언트 필터(B안) 수정 완료 (260428 코드 탐색 확인)
**증상**: 즐겨찾기·스크랩·오답 화면이 전체 목록만 표시. 태그/키워드 필터 없어 콘텐츠 누적 시 탐색 불편
**원인**: 피드에는 태그 칩 필터가 있지만 즐겨찾기·오답 화면에 동일 컴포넌트 미적용

**수정 완료**:
- 웹: `favorites-filter.ts` `filterWrongAnswers()` + `+page.svelte` `$derived` 변수
- iOS: `WrongAnswerTagFilter.swift` 순수 함수 + `FavoritesView.swift` computed 프로퍼티

**미해결 부채**: DEBT-01 — 서버 `wrong_answers.tag_id` 컬럼 추가(A안) 보류. 현재 `favorites.tag_id` 브릿지 방식(B안) 동작 중.

**우선순위**: MVP11 기능 개선 항목

---

## [BUG-005] 즐겨찾기 삭제 후 고아 selectedTagId — 빈 목록 고정

**발견**: 2026-04-24 (MVP11 M4 리뷰 중)
**상태**: ✅ **RESOLVED** (260428 코드 탐색 확인)
**플랫폼**: iOS + Web 공통

**수정 완료**:
- iOS: `FavoritesFeature.shouldResetTagId(remaining:current:)` 정적 메서드 + `FavoritesView` swipeActions 삭제 핸들러에서 호출
- Web: `favorites-filter.ts`의 `shouldResetTagId` 유틸 + `+page.svelte` `handleRemoveFavorite`에서 호출

**상세**: `progress/bugs/BUG-005_orphan-selectedTagId-after-delete.md`

---

## [BUG-006] 요약 실패 시 에러 상태 캐싱 — 재시도 불가

**발견**: 2026-04-29 실사용 테스트 중
**상태**: ✅ **RESOLVED** — MVP14 M2 ST-03/ST-04 코드 탐색 완료, 에러 캐시 코드 없음 확인
**플랫폼**: iOS + Web 공통 (추정)
**재현**: 특정 기사에서 요약하기 실패 → 다시 시도 버튼 → 계속 실패. 다른 기사 탐색 후 복귀해도 동일.
**증상**: 한 번 실패한 기사는 이후 시도에서도 계속 실패. API 사용량 초과는 아닌 것으로 추정.
**실제 원인**: 코드 탐색 결과 iOS `SummarySessionCache`, 웹 `summaryCache` 모두 에러를 캐시에 저장하지 않음.
"계속 실패" 증상은 클라이언트 에러 캐시 버그가 아닌 서버/외부 크롤러(crawl/LLM) 지속 장애로 인한 반복 실패. 재시도 시 API 재호출은 정상 동작.
**서버 부분 실패 수정**: 피드 캐시에서 부분 실패(일부 태그 검색 오류) 시 5분 TTL → 1분 단축 TTL 적용 (F-03).
**우선순위**: MVP14 M2

---

## [BUG-007] 태그별 당겨서 새로고침이 실제 반영 안 됨

**발견**: 2026-04-29 실사용 테스트 중
**상태**: ✅ **RESOLVED** — MVP14 M2 ST-05 코드 탐색 완료, 정상 동작 확인
**플랫폼**: iOS
**증상**: 특정 태그 탭에서 pull-to-refresh 실행 시 기사 목록이 변하지 않음. 새로고침이 실제로 동작하는지 불명확.
**실제 원인**: `FeedFeature.refresh()`는 `noCache: true`로 서버 캐시 우회 + `rebuildTagStates()` 전체 재구성. 코드상 정상 동작. "변하지 않음"은 새 기사가 아직 없어서 동일하게 보이는 정상 동작일 가능성이 높음.
**우선순위**: MVP14 M2

---

## [BUG-008] 태그 탭 전환 시 "기사가 없습니다" 깜빡임

**발견**: 2026-04-29 실사용 테스트 중
**상태**: ✅ **RESOLVED** — MVP14 M2 ST-06/ST-07 수정 완료
**플랫폼**: iOS + Web 공통

**수정 완료**:
- iOS: `FeedFeature.isLoading`에 `tagStates[currentKey]?.status == .loading` 조건 추가. 탭 전환 캐시 미스 시 ShimmerListView 표시 → EmptyStateView 깜빡임 제거.
- 웹: `feedStore.selectTag()` 내 `tagCache` 업데이트를 `activeTagId` 변경보다 먼저 수행. `isTagLoading=true` 상태로 activeTagId가 바뀌도록 순서 수정. 에러 시 `{status: 'error'}` 캐시 저장 제거 → 키 삭제로 재시도 가능.

**우선순위**: MVP14 M2

---

## [BUG-009] 특정 기사 썸네일이 동일 이미지로 표시

**발견**: 2026-04-29 실사용 테스트 중
**상태**: ✅ **RESOLVED** — 외부 의존 한계로 수정 불가, 원인 확정 후 문서화 완료 (260429)
**플랫폼**: iOS + Web 공통
**증상**: 여러 기사의 썸네일이 동일한 이미지로 표시됨.
**실제 원인 (260429 실기기 확인)**: 동일 뉴스 이벤트(예: 네이버 웨일 멀티플레이 출시)를 여러 언론사가 보도할 때, 각 언론사가 동일한 보도자료 이미지를 `og:image`로 사용. 크롤링 코드는 정상 동작 중이며 코드 버그 아님.
**한계**: 외부 사이트 `og:image` 정책에 따른 문제로 서버에서 근본 해결 불가. 동일 이벤트 기사가 몰릴 때마다 자연 발생.
**우선순위**: 해결 없이 종료 (외부 의존)

---

## [BUG-010] 태그 전환 시 기사 자동 변경 동작 방식 미검증

**발견**: 2026-04-29 실사용 테스트 중
**상태**: ✅ **RESOLVED** — MVP14 M2 ST-08 코드 탐색 완료, 정상 동작 확인
**플랫폼**: iOS + Web
**증상**: 태그를 누르다 보면 기사 목록이 자동으로 바뀜. 의도된 동작인지 불명확.
**판단**: 정상 동작. 탭 선택은 명시적 액션이며, 캐시 미스 시 서버 재요청 → 최신 기사 표시는 의도된 설계.
상세: `progress/mvp14/BUG-010_analysis.md`
**우선순위**: MVP14 M2

---

## [BUG-004] 기사 목록 인덱스 페이지가 기사로 수집됨

**발견**: 2026-04-23 웹 테스트 계정 확인 중
**상태**: ✅ **RESOLVED** — MVP12 M1에서 수정 완료 (260428 코드 탐색 확인)
**증상**: 개별 기사 대신 뉴스 카테고리/태그 인덱스 페이지가 피드에 노출됨.
기사 소개글이 Sentry JS 코드·내비게이션 텍스트·기사 제목 나열로 표시됨
예: "Data science recent news | AI Business", "data science News & Articles - IEEE Spectrum"

**수정 완료**:
- `server/src/api/feed.rs` — `is_listing_url()` 3규칙 후처리 (모든 엔진 통합 적용)
- `server/src/infra/tavily.rs` — `"topic": "news"`, `"time_range": "week"` 파라미터 추가
- `server/src/infra/exa.rs` — `"category": "news"`, `"startPublishedDate"` 파라미터 추가

**우선순위**: MVP11 기능 개선 항목
