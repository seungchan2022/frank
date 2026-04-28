# 알려진 버그 목록

발견된 버그를 기록. 다음 MVP 기능 구현 전에 수정 후 진행.

> 260428 상태 정리: BUG-001~004 모두 코드에서 수정 완료. 상세 분석: `progress/analysis/260428_full_debt-bugs.md`

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
