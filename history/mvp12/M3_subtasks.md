# MVP12 M3 서브태스크 목록

> 생성일: 260427
> 마일스톤: M3 — iOS 버그 수정 + UX 개선 (M2 웹 싱크)
> 브랜치: feature/260427_mvp12-m3-ios-bug-ux
> 상태: in-progress

## 개요

M2(웹) 확정 기능 범위와 동일한 커버리지로 iOS 구현.
BUG-D·BUG-E·BUG-F 수정, 좋아요/즐겨찾기 UX 재설계, 무한 스크롤 도입.
수정 대상 앱: iOS(SwiftUI/Tuist).

---

## 싱크 기준 (M2 웹 확정본)

| 항목 | 웹 확정 | iOS 현재 | 갭 |
|---|---|---|---|
| BUG-D | `$derived` 자동 반영 | `hasLoaded` 가드로 재호출 차단 | 태그 미갱신 |
| BUG-E | `shouldResetTagId` 후 null | 초기화 없음 | 먹통 버그 |
| BUG-F 필터 | favorites 미등록 오답 → 제외 | 항상 포함 | 정책 반대 |
| BUG-F 칩 | wrongAnswerFilterTags (오답-즐겨찾기 교집합) | feature.tags (즐겨찾기 전체) | 칩 소스 오류 |
| 좋아요 레이블 | "추천에 반영" / "추천 완료" | "좋아요" / "좋아요 완료" | 레이블 불일치 |
| 즐겨찾기 레이블 | 🔖 "스크랩 저장" / "스크랩 해제" | ★ "즐겨찾기 추가" / "즐겨찾기 해제" | 레이블·아이콘 불일치 |
| 무한 스크롤 | IntersectionObserver + loadMore() | 미구현 | 신규 기능 |

---

## 서브태스크 목록

### ST1 — BUG-D: addFavorite/removeFavorite 후 tags 로컬 갱신

**목적**: 즐겨찾기 추가/삭제 시 스크랩 탭 태그 칩 즉시 갱신.

**문제 분석**:
- `FavoritesFeature.addFavorite()` (FavoritesFeature.swift:124): `items`에 prepend는 하지만 `tags` 갱신 없음.
- `FavoritesFeature.removeFavorite()` (FavoritesFeature.swift:166): `items`에서 제거만 하고 `tags` 갱신 없음.
- `loadFavorites()` (FavoritesFeature.swift:85): `hasLoaded=true`이면 no-op → 재호출로도 tags 안 갱신됨.
- **tags가 items와 동기화 안 됨**: 새 기사 추가 시 칩 미등장, 마지막 기사 삭제 시 stale 칩 잔존.

**구현**:
- `FavoritesFeature`에 `recomputeTags()` private 메서드 추가:
  - `items`에서 tagId Set 추출 → `allTagsCache`와 교집합 → `tags` 갱신.
  - `allTagsCache`는 `loadFavorites` 시 `fetchAllTags()` 결과를 저장해두고 재사용 (`private var allTagsCache: [Tag] = []`).
  - `fetchAllTags()` 재호출 없이 items 기반으로 재계산 → 네트워크 비용 없음.
- `addFavorite` 성공 후 `recomputeTags()` 호출.
- `removeFavorite` 성공 후 `recomputeTags()` 호출 (step-5 리뷰: removeFavorite 후 stale 태그 칩 잔존 수정).
- `hasLoaded` 가드는 그대로 유지.

**수정 파일**:
- `ios/Frank/Frank/Sources/Features/Favorites/FavoritesFeature.swift`

**산출물**:
- 유닛 테스트: addFavorite 후 새 tagId가 tags에 반영됨 (1건)
- 유닛 테스트: removeFavorite 후 마지막 기사 제거 시 해당 태그가 tags에서 제거됨 (1건) [step-5 추가]

**의존성**: 없음
**예상 소요**: ~30m

---

### ST2 — BUG-E: removeFavorite 후 selectedTagId nil 초기화

**목적**: 마지막 기사 삭제 후 스크랩 탭 빈 화면 고정 방지.

**문제 분석**:
- `FavoritesFeature.removeFavorite()` (FavoritesFeature.swift:166): `items`에서 제거만 함.
- `selectedTagId`는 `FavoritesView`의 `@State` → Feature에서 직접 변경 불가.
- 남은 items에 현재 selectedTagId 매칭 항목이 없으면 filteredItems = [] → 빈 화면 고정.

**구현**:
- `FavoritesFeature`에 `shouldResetTagId(remaining: [FavoriteItem], current: UUID?) -> Bool` 순수 함수 추가.
- `FavoritesView` itemList의 swipeActions 핸들러에서 `removeFavorite` 완료 후 `shouldResetTagId` 판단 → `selectedTagId = nil` 처리.
  - 웹 BUG-C와 동일 로직: 같은 태그의 다른 기사가 남아있으면 초기화 안 함.

**수정 파일**:
- `ios/Frank/Frank/Sources/Features/Favorites/FavoritesFeature.swift`
- `ios/Frank/Frank/Sources/Features/Favorites/FavoritesView.swift`

**산출물**:
- 유닛 테스트: shouldResetTagId 회귀 테스트 (BUG-E 시나리오 5건)

**의존성**: 없음
**예상 소요**: ~30m

---

### ST3 — BUG-F iOS: WrongAnswerTagFilter 정책 + 오답 칩 소스 교체

**목적**: 오답 탭 태그 필터 미작동 수정 + M2 웹 정책 싱크.

**문제 분석**:
- **문제 1 — 필터 정책**: `WrongAnswerTagFilter.apply()` (WrongAnswerTagFilter.swift:44) "favorites에 없는 오답은 항상 표시" → M2 웹은 "제외".
- **문제 2 — 칩 소스**: `FavoritesView.tagChipBar`가 기사·오답 탭 공용 `feature.tags`(즐겨찾기 전체 태그) 사용 → 오답에 없는 태그 칩도 노출.

**구현**:
1. `WrongAnswerTagFilter.apply()`: `tagMap[wrongAnswer.articleUrl] == nil` 시 `return true` → `return false` 변경.
2. `FavoritesView`에 `wrongAnswerTags: [Tag]` computed 추가:
   - **소스 정정 (step-5 리뷰 반영)**: `buildTagMap` tagId Set이 아닌 `wrongAnswersFeature.items`에서 직접 tagId 추출.
   - 계산식: `wrongAnswersFeature.items` → `articleUrl` → `wrongAnswerTagMap` 조인 → tagId Set → `feature.tags` 교집합.
   - 즉: `let tagIds = Set(wrongAnswersFeature.items.compactMap { wrongAnswerTagMap[$0.articleUrl] }); return feature.tags.filter { tagIds.contains($0.id) }`
   - 이유: buildTagMap 기반 접근은 "오답이 없는 태그"도 포함해 빈 결과 칩 노출 버그를 유발.
3. `wrongAnswersContent`의 `tagChipBar` 호출을 `wrongAnswerTags` 기준으로 분기.
4. **selectedTagId 무효화 처리 추가 (step-5 리뷰 반영)**: `wrongAnswerTags`가 변경될 때 `selectedTagId`가 새 집합에 없으면 nil 초기화. SwiftUI `.onChange(of: wrongAnswerTags)`로 처리.

**수정 파일**:
- `ios/Frank/Frank/Sources/Features/Favorites/WrongAnswerTagFilter.swift`
- `ios/Frank/Frank/Sources/Features/Favorites/FavoritesView.swift`

**산출물**:
- 유닛 테스트 업데이트: `WrongAnswerTagFilterTests` — "favorites 미등록 오답 → 제외" 케이스 추가, 기존 "항상 표시" 케이스 제거
- 유닛 테스트: `wrongAnswerTags` computed 검증 (오답 있는 태그만 반환, 오답 없는 태그 미포함)

**의존성**: 없음
**예상 소요**: ~45m

---

### ST4 — 좋아요/즐겨찾기 UX 재설계 (M2 레이블·아이콘 싱크)

**목적**: 좋아요·즐겨찾기 역할 구분을 iOS에서도 M2 웹과 동일하게 표현.

**M2 웹 확정 UX**:
- 좋아요(피드 카드): `♥/♡` + "추천에 반영" / "추천 완료"
- 즐겨찾기(기사 상세): 🔖 + "스크랩 저장" / "스크랩 해제"
- 즐겨찾기 해제(favorites 페이지): 🔖 "스크랩 해제"

**iOS 수정 내용**:
1. `ArticleDetailView` 좋아요 버튼 레이블:
   - `"좋아요"` → `"추천에 반영"`, `"좋아요 완료"` → `"추천 완료"`
   - 아이콘: `heart`/`heart.fill` 유지
2. `ArticleDetailView` 즐겨찾기 버튼 레이블·아이콘:
   - `"즐겨찾기 추가"` → `"스크랩 저장"`, `"즐겨찾기 해제"` → `"스크랩 해제"`
   - 아이콘: `star`/`star.fill` → `bookmark`/`bookmark.fill`
3. `FavoritesView` swipeActions 삭제 레이블은 유지 (이미 "삭제"로 적절)

**수정 파일**:
- `ios/Frank/Frank/Sources/Features/Detail/ArticleDetailView.swift`
- `ios/Frank/Frank/Sources/Features/Feed/FeedView.swift` [step-5 추가 — FeedView:124 accessibilityLabel 동기화]

**산출물**:
- 시뮬레이터 스크린샷: 기사 상세 버튼 영역 (변경 전/후)

**의존성**: 없음
**예상 소요**: ~20m

---

### ST5 — 무한 스크롤: ArticlePort 확장 + FeedFeature loadMore + FeedView sentinel

**목적**: 피드 리스트 하단 도달 시 다음 페이지 자동 로드 (M2 웹 offset 전략 싱크).

**구현 단계**:

#### 5-1. ArticlePort + 어댑터 확장
- `ArticlePort.fetchFeed` 시그니처:
  ```swift
  func fetchFeed(tagId: UUID?, noCache: Bool, limit: Int? = nil, offset: Int? = nil) async throws -> [FeedItem]
  ```
- `APIArticleAdapter.fetchFeed`: `limit`, `offset` 있으면 query parameter 추가
- `MockArticleAdapter.fetchFeed`: 파라미터 추가 (테스트용 stub, 파라미터 무시 가능)

#### 5-2. FeedFeature 페이지네이션 상태 추가
- `TagState` 타입 도입:
  ```swift
  struct TagState {
      var items: [FeedItem] = []
      var nextOffset: Int = 0
      var hasMore: Bool = true
      var status: TagStatus = .idle
  }
  enum TagStatus { case idle, loading, loadingMore, error }
  ```
- `tagCache: [String: [FeedItem]]` → `tagStates: [String: TagState]`로 교체
- `feedItems`는 `tagStates[currentKey]?.items ?? []` 투영
- **TagState Dictionary 변이 패턴 (step-5 리뷰 반영)**: `@Observable` + `Dictionary<String, struct>` 변이는 반드시 read-modify-write 패턴 사용:
  ```swift
  var state = tagStates[key] ?? TagState()
  state.items.append(contentsOf: newItems)
  state.nextOffset += newItems.count
  state.hasMore = newItems.count >= PAGE_SIZE
  tagStates[key] = state  // 명시적 재할당 필수
  ```
- `loadMore()` 함수 추가:
  - `hasMore=false` 또는 `status=loadingMore` 시 재진입 차단
  - `offset` 캡처 후 fetch → items append, nextOffset 갱신, PAGE_SIZE(20) 미만 시 `hasMore=false`
- `loadInitial`: 전체 탭만 즉시 로드 (M2 웹 C안 — 구독 태그 병렬 프리패치 제거)
- `selectTag`: 캐시 미스 시 lazy fetch (첫 페이지)
- `refresh()`: 현재 탭 `TagState` 리셋 후 첫 페이지 재요청
- PAGE_SIZE 상수: 20

#### 5-3. FeedView sentinel 추가
- 리스트 마지막 아이템 뒤에 빈 `Color.clear` sentinel Row 추가
- `.onAppear` → `Task { await feedFeature.send(.loadMore) }` 호출 (onAppear는 sync — Task 래핑 필수)
- **중복 발화 방지 (step-5 리뷰 반영)**: `@State private var loadMoreTask: Task<Void, Never>?` 관리. sentinel onAppear 시 이전 task를 cancel 후 새 Task 할당. FeedFeature 내 `status == .loadingMore` guard는 보조 방어선으로 유지.
- `hasMore=false` 시 sentinel 대신 "모든 기사를 읽었습니다" Text 표시
- `status=loadingMore` 시 `ProgressView()` 표시 (sentinel 위)

**수정 파일**:
- `ios/Frank/Frank/Sources/Core/Ports/ArticlePort.swift`
- `ios/Frank/Frank/Sources/Core/Adapters/APIArticleAdapter.swift`
- `ios/Frank/Frank/Sources/Core/Adapters/MockArticleAdapter.swift`
- `ios/Frank/Frank/Sources/Features/Feed/FeedFeature.swift`
- `ios/Frank/Frank/Sources/Features/Feed/FeedView.swift`

**산출물**:
- 유닛 테스트: loadMore() 정상 누적, hasMore=false 정지, 중복 가드
- 유닛 테스트: selectTag — 미방문 탭 최초 fetch, 재방문 캐시 히트

**의존성**: 없음 (ST1~ST4와 병렬 가능)
**예상 소요**: ~2h

---

## 의존성 그래프

```
ST1 ──┐
ST2 ──┤
ST3 ──┼── (병렬) ── 전체 완료 후 ST6(통합 테스트·검증)
ST4 ──┤
ST5 ──┘
```

---

## KPI 체크포인트

| 지표 | 기준 | 게이트 |
|---|---|---|
| iOS 테스트 커버리지 | ≥85% (M3 게이트). swift-scope 90% 기준은 MVP12 최종 KPI 시점 적용 | Soft |
| BUG-D 재현 0 | 즐겨찾기 추가 후 태그 칩 표시 확인 | Hard |
| BUG-E 재현 0 | 마지막 기사 삭제 후 전체 탭 표시 확인 | Hard |
| BUG-F 재현 0 | 오답 탭 태그 필터 동작 확인 | Hard |
| 무한 스크롤 정상 동작 | 시뮬레이터 수동 확인 | Hard |

---

## OPEN QUESTIONS (인터뷰 완료)

| 번호 | 질문 | 결정 |
|---|---|---|
| OQ1 | BUG-F iOS 처리 방향 | 웹 현재 상태 싱크 (즐겨찾기 미등록 오답 → 제외) |
| OQ2 | 구현 순서 | 병렬 분해 후 한번에 구현 |
| OQ3 | ArticlePort 변경 범위 | Optional 파라미터 추가 (하위 호환 유지) |

---

## Feature List
<!-- size: 중형 | count: 45 | skip: false | all-checked: false -->

### 기능
- [x] F-01 BUG-D: FavoritesFeature에 recomputeTags() 추가 — addFavorite/removeFavorite 후 items 기반 tags 재계산 (allTagsCache 재사용, fetchAllTags 재호출 없음)
- [x] F-02 BUG-E: removeFavorite 성공 후 shouldResetTagId 판단 → selectedTagId nil 초기화
- [x] F-03 BUG-F: WrongAnswerTagFilter.apply() favorites 미등록 오답 → 제외 (return false)
- [x] F-04 BUG-F: FavoritesView wrongAnswerTags computed 추가 (wrongAnswersFeature.items → tagMap 조인 → feature.tags 교집합) [소스 정정]
- [x] F-05 BUG-F: wrongAnswersContent tagChipBar 소스를 wrongAnswerTags로 교체
- [x] F-06 UX: ArticleDetailView 좋아요 레이블 "추천에 반영" / "추천 완료"
- [x] F-06b UX: FeedView 피드 카드 좋아요 accessibilityLabel "추천에 반영" / "추천 완료" [step-5 추가]
- [x] F-07 UX: ArticleDetailView 즐겨찾기 레이블 "스크랩 저장" / "스크랩 해제" + bookmark 아이콘
- [x] F-08 ArticlePort.fetchFeed에 limit?/offset? Optional 파라미터 추가
- [x] F-09 APIArticleAdapter: limit/offset query parameter 전달
- [x] F-10 FeedFeature: TagState 타입 도입 (items, nextOffset, hasMore, status)
- [x] F-11 FeedFeature: tagCache → tagStates 교체, feedItems $derived 투영
- [x] F-12 FeedFeature: loadInitial — 전체 탭만 즉시, 구독 태그 프리패치 제거
- [x] F-13 FeedFeature: selectTag — 캐시 미스 시 lazy fetch 첫 페이지
- [x] F-14 FeedFeature: loadMore() — 현재 탭 nextOffset 기준 다음 페이지 append
- [x] F-15 FeedFeature: refresh() — 현재 탭 TagState 리셋 후 첫 페이지 재요청
- [x] F-16 FeedView: sentinel onAppear → loadMore() 호출
- [x] F-17 FeedView: loadingMore 스피너 + hasMore=false "모든 기사를 읽었습니다" 메시지

### 엣지
- [x] E-01 shouldResetTagId: 같은 태그의 다른 기사 남아있으면 초기화 안 함
- [x] E-02 loadMore: hasMore=false 또는 status=loadingMore 시 재진입 차단
- [x] E-03 loadMore: PAGE_SIZE(20) 미만 응답 시 hasMore=false 즉시 설정
- [x] E-04 selectTag: lazy fetch 중 key 먼저 캡처 후 await (다른 탭 전환 시 결과 올바른 key에 기록)
- [x] E-05 wrongAnswerTags: wrongAnswers 빈 배열일 때 칩 미표시
- [x] E-06 sentinel 중복 발화: onAppear 재호출 시 이전 loadMoreTask cancel 후 재생성 [step-5 추가]
- [x] E-07 recomputeTags: addFavorite/removeFavorite 후 tags가 items와 동기화됨 (allTagsCache 기반) [step-5 추가]
- [x] E-08 wrongAnswerTags 변경 시 selectedTagId 무효화 — 새 집합에 없는 태그 선택 시 nil 초기화 [step-5 추가]

### 에러
- [x] R-01 loadMore API 실패 시 status=error, 에러 메시지 표시
- [x] R-02 addFavorite 후 fetchAllTags 실패 시 tags 변경 없음 (폴백)

### 테스트
- [x] T-01 FavoritesFeature 유닛: addFavorite 후 새 tagId tags 반영 (recomputeTags 호출)
- [x] T-01b FavoritesFeature 유닛: removeFavorite 후 마지막 기사 삭제 시 해당 태그 tags에서 제거 [step-5 추가]
- [x] T-02 FavoritesFeature 유닛: shouldResetTagId 회귀 5건 (BUG-E 시나리오)
- [x] T-03 WrongAnswerTagFilterTests 업데이트: favorites 미등록 오답 → 제외 / wrongAnswerTags 소스 검증 (오답 없는 태그 미포함)
- [x] T-04 FeedFeature 유닛: loadMore() 정상 누적 (read-modify-write 패턴 검증), hasMore=false 정지, 중복 가드, 여러 페이지 누적 후 캐시 키 일치
- [x] T-05 FeedFeature 유닛: selectTag 캐시 미스 fetch, 캐시 히트 (fetch 없음)
- [x] T-06 FeedFeature 유닛: loadMore API 실패 시 status=error (R-01)
- [x] T-07 FavoritesFeature 유닛: addFavorite 후 fetchAllTags 실패 시 tags 변경 없음 (R-02)
- [x] T-08 APIArticleAdapter 유닛: limit/offset 있을 때 query parameter 포함 확인
- [x] T-09 ArticleDetailView 레이블 검증: U-01 E2E(시뮬레이터)로 대체 완료 (ViewInspector 미사용)

### UI·UX
- [x] U-01 시뮬레이터 E2E: 기사 상세 좋아요("추천에 반영") · 즐겨찾기("스크랩 저장") 레이블·아이콘 확인
- [x] U-02 시뮬레이터 E2E: 피드 무한 스크롤 동작 확인
- [ ] U-03 시뮬레이터 E2E: 오답 탭 태그 칩 (wrongAnswerTags 기준) 확인 — defer (오답노트 태그 칩 근본 수정은 다음 MVP)

### 회귀
- [x] G-01 FeedFeature.refresh() 동작 유지 — 현재 탭 초기화 + 새 페이지 로드
- [x] G-02 FeedFeature.reloadAfterTagChange() 동작 유지
- [x] G-03 FavoritesFeature.filteredItems() 기존 동작 유지
- [x] G-04 WrongAnswerTagFilter.buildTagMap() 기존 동작 변경 없음

---

## Step-5 리뷰 결과 (260427)

### Claude 리뷰 (문서 일치성)

**치명**: ST3 wrongAnswerTags 소스 모순 — buildTagMap 기반은 "오답 없는 태그"도 포함.
**치명**: ST5 sentinel onAppear 중복 발화 — Task 중복 가드 미명시.
**중대**: ST1/ST2 removeFavorite 후 tags stale — add만 다루고 remove 누락.
**중대**: ST5 TagState Dictionary 변이 반응성 미보장 — read-modify-write 패턴 미명시.
**중대**: FeedView 좋아요 accessibilityLabel 누락 — ST4 수정 파일 목록에 FeedView.swift 미포함.
**중대**: BUG-F 수정 후 오답 탭 selectedTagId 무효화 미처리.
**중대**: ST5 loadInitial 프리패치 제거 후 캐시 미스 탭 전환 시 로딩 표시 없음.

### Codex 리뷰 (기술적 타당성)

- ST1: @Observable + @MainActor fetchAllTags 재호출 패턴 기술적으로 OK. 단, allTagsCache 기반 recomputeTags 방식 권장.
- ST2: shouldResetTagId를 Feature에 추가 + View에서 호출은 중간지대. 현재 selectedTagId가 View 소유이므로 View 판단 자체는 수용 가능.
- ST3: wrongAnswerTags 소스 오류 확인 — wrongAnswersFeature.items 기반 추출 필요.
- ST5: .onAppear Task 래핑 필수. Dictionary<String, struct> 변이는 read-modify-write 필수. 서버 limit/offset 지원 사전 확인 필요.

### 최종 결정: 조건부 승인

치명 2건, 중대 5건 수정 반영 후 구현 착수.
수정 내용: wrongAnswerTags 소스 정정, sentinel 중복 가드 명시, recomputeTags 도입, TagState 변이 패턴 명시, FeedView.swift ST4 수정 파일 추가, 오답 탭 selectedTagId 무효화 처리.
경미 4건(T1 CacheKey 타입, T2 레이어 경계 이슈, T3 heart/bookmark 아이콘 일치, T4 ViewInspector 호환성)은 후속 과제로 기록.
