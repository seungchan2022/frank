# ST-2: FavoritesView 기사 탭 태그 칩 필터 UI 추가

> 의존: ST-1 + ST-5 완료 후 진행

---

## 수정 대상 파일

| 파일 | 변경 종류 |
|---|---|
| `ios/Frank/Frank/Sources/Features/Favorites/FavoritesView.swift` | 기사 탭 UI에 TagChipBarView 추가 |

---

## 변경 내용

### FavoritesView.swift — articlesContent .done 케이스

1. `TagChipBarView` 추가 (기사 목록 위)
   - `tags`: `feature.tags` 전달
   - `selectedTagId`: `feature.selectedTagId` 전달
   - `onSelect`: `Task { await feature.selectTag($0) }` 호출
2. 기사 목록 데이터 소스를 `feature.items` → `feature.filteredItems` 로 변경

### 레이아웃 순서

```
[세그먼트 컨트롤]
[TagChipBarView — 가로 스크롤]
[Divider]
[기사 목록 (filteredItems 기반)]
```

---

## 완료 조건

- `xcodebuild build` → BUILD SUCCEEDED
- `FavoritesViewTests` 칩 바 렌더 테스트 그린:
  - `feature.tags`가 비어있지 않을 때 `TagChipBarView` 노출 확인
- 기존 `FavoritesView` 관련 테스트 전체 그린 유지 (회귀 없음)

---

## 참조

- `ios/Frank/Frank/Sources/Components/TagChipBar/` (재사용 컴포넌트)
- `ios/Frank/Frank/Sources/Features/Feed/FeedView.swift` (패턴 참조)
