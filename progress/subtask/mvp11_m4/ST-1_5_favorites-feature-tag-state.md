# ST-1 + ST-5: FavoritesFeature 태그 상태 추가 + DI 주입 갱신

> 묶음 처리 이유: ST-1에서 `FavoritesFeature.init` 시그니처가 변경되므로,
> ST-5(DI 갱신)를 동시에 처리해야 컴파일이 통과된다.

---

## 수정 대상 파일

| 파일 | 변경 종류 |
|---|---|
| `ios/Frank/Frank/Sources/Features/Favorites/FavoritesFeature.swift` | 상태 추가 + init 시그니처 변경 |
| 앱 DI 조립 위치 (RootView / MainTabView 등) | TagAdapter 주입 |

---

## 인터뷰 확정 사항

- 태그 로딩: `fetchAllTags()` 만 사용. `items`에 실제 존재하는 tagId 교집합만 칩으로 표시 (웹 싱크)
- 병렬 호출: `async let favorites` + `async let allTags` 동시 실행
- `fetchMyTagIds()` 호출 없음

## 변경 내용

### FavoritesFeature.swift

1. `init(favorites: any FavoritesPort)` → `init(favorites: any FavoritesPort, tag: any TagPort)` 로 변경
2. `private(set) var tags: [Tag] = []` 상태 추가
3. `private(set) var selectedTagId: UUID? = nil` 상태 추가
4. `load()` 내 `async let` 병렬 호출:
   ```swift
   let fetchedItems = try await favorites.listFavorites()
   self.items = fetchedItems
   let favTagIds = Set(fetchedItems.compactMap(\.tagId))
   tags = (try? await tag.fetchAllTags())?.filter { favTagIds.contains($0.id) } ?? []
   ```
5. `func selectTag(_ tagId: UUID?)` 액션 추가
6. `var filteredItems: [FavoriteItem]` computed 프로퍼티 추가
   - `selectedTagId == nil` → `items` 전체 반환
   - `selectedTagId != nil` → `items.filter { $0.tagId == selectedTagId }`

### FeedFeature.swift (별도 perf 커밋)

- `loadInitial()` 내 순차 호출 → `async let` 병렬 호출로 개선

### DI 조립 위치 (FrankApp.swift)

- `FavoritesFeature(favorites: favoritesPort)` → `FavoritesFeature(favorites: favoritesPort, tag: dependencies.tag)` 로 변경

---

## 완료 조건

- `xcodebuild build` → BUILD SUCCEEDED
- `FavoritesFeatureTests` 신규 케이스 3개 모두 그린:
  - `selectTag(nil)` → `filteredItems` 전체 반환
  - `selectTag(tagId)` → 해당 tagId 아이템만 반환
  - `selectTag(nil)` 재선택 → 전체 복원

---

## 참조

- `ios/Frank/Frank/Sources/Core/Ports/TagPort.swift`
- `ios/Frank/FrankTests/Mocks/MockTagPort.swift`
- `ios/Frank/Frank/Sources/Features/Feed/FeedFeature.swift` (패턴 참조)
