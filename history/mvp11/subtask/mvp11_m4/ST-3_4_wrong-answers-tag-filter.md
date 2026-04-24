# ST-3 + ST-4: WrongAnswers 태그맵 + 오답 탭 태그 칩 UI 추가

> 묶음 처리 이유: ST-3(뷰 레벨 computed 상태)과 ST-4(UI 연결)이 동일 파일(FavoritesView.swift) 내에 있어
> 함께 처리하는 것이 자연스럽다.
> 의존: ST-1 + ST-5 완료 후 진행 (ST-2와 병렬 가능)

---

## 수정 대상 파일

| 파일 | 변경 종류 |
|---|---|
| `ios/Frank/Frank/Sources/Features/Favorites/FavoritesView.swift` | WrongAnswers 필터 상태 + 오답 탭 UI |

---

## 인터뷰 확정 사항

- `selectedTagId`: 기사·오답 탭 **공유** `@State` 하나 (웹 싱크)
- 탭 전환 시 `selectedTagId = nil` 초기화
- tagMap은 View 레벨 computed (Feature 간 결합 없음)

## 변경 내용 (ST-3: 뷰 레벨 상태 추가)

### FavoritesView.swift — 프로퍼티 추가

1. `@State private var selectedTagId: UUID? = nil` — 기사·오답 탭 공유 (ST-2에서도 사용)
2. `var wrongAnswerTagMap: [String: UUID]` computed 프로퍼티 추가
   - `feature.items`에서 `url → tagId` 매핑 (tagId != nil 인 것만)
3. `var filteredWrongAnswers: [WrongAnswer]` computed 프로퍼티 추가
   - `selectedTagId == nil` → `wrongAnswersFeature.items` 전체
   - `selectedTagId != nil`:
     1. `wrongAnswerTagMap`에서 해당 tagId의 url Set 추출
     2. `wrongAnswersFeature.items.filter { urlSet.contains($0.articleUrl) || wrongAnswerTagMap[$0.articleUrl] == nil }`

> articleUrl이 favorites에 없는 오답은 어떤 태그 선택에도 항상 표시 (웹 동작과 동일).
> 아키텍처 경계: 표현 로직(뷰 레벨 private computed)으로 허용.

---

## 변경 내용 (ST-4: UI 추가)

### FavoritesView.swift — wrongAnswersContent .done 케이스

1. `TagChipBarView` 추가 (오답 목록 위)
   - `tags`: `feature.tags` 전달 (기사 탭과 동일한 태그 목록)
   - `selectedTagId`: 공유 `selectedTagId` 전달
   - `onSelect`: `selectedTagId = $0` (동기 상태 업데이트)
2. 오답 목록 데이터 소스를 `wrongAnswersFeature.items` → `filteredWrongAnswers` 로 변경

---

## 완료 조건

- `xcodebuild test` → TEST SUCCEEDED (전체)
- `FavoritesViewTests` 필터 로직 테스트 그린:
  - `wrongAnswersTags`: FavoriteItems에 tagId가 있을 때 해당 Tag만 반환
  - `filteredWrongAnswers(nil)`: 전체 WrongAnswer 반환
  - `filteredWrongAnswers(tagId)`: url Set 교집합 기반 필터 결과 확인
- `WrongAnswersFeatureTests` 기존 테스트 전체 그린 유지 (회귀 없음)

---

## 참조

- `ios/Frank/Frank/Sources/Core/Models/WrongAnswer.swift` (`articleUrl: String`)
- `ios/Frank/Frank/Sources/Core/Models/FavoriteItem.swift` (`tagId: UUID?`, `url: String`)
- `ios/Frank/FrankTests/Features/Favorites/WrongAnswersFeatureTests.swift` (회귀 방지)
