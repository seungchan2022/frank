# BUG-004: 즐겨찾기 삭제 후 고아 selectedTagId — 빈 목록 고정

> 발견: 2026-04-24 (MVP11 M4 리뷰 중)
> 플랫폼: iOS + Web (공통)
> 심각도: Medium (특정 조건에서만 발생, 앱 크래시 없음)

## 증상

특정 태그 칩 선택 중 해당 태그의 마지막 즐겨찾기를 삭제하면
칩은 사라지지만 `selectedTagId`는 그대로 남아 목록이 빈 화면으로 고정된다.
사용자가 다른 칩을 선택할 수도 없다 — 해당 칩이 이미 사라졌기 때문.

## 재현 시나리오

1. 즐겨찾기에 AI 태그 기사 1개, iOS 태그 기사 2개가 있는 상태
2. 태그 칩 "AI" 선택 → AI 기사 1개 표시
3. 해당 AI 기사 즐겨찾기 해제
4. **기대**: 전체 목록으로 자동 복귀 또는 "AI" 칩 사라지며 iOS 기사 표시
5. **실제**: 칩 바에서 "AI" 사라짐 + `selectedTagId = AI UUID` 유지 → 빈 화면 고정

## 영향 범위

- iOS `FavoritesView` 기사 탭
- iOS `FavoritesView` 오답 탭 (동일 `selectedTagId` 공유)
- Web `favorites/+page.svelte` (동일 버그, 미처리)

## 수정 방향

즐겨찾기 삭제 후 `tags` 재계산 시 `selectedTagId`가 새 목록에 없으면 `nil`로 초기화.

### iOS
`FavoritesFeature.removeFavorite()` 완료 후:
```swift
if let id = selectedTagId, !tags.contains(where: { $0.id == id }) {
    selectedTagId = nil
}
```

### Web (`favorites/+page.svelte`)
`handleRemoveFavorite()` 완료 후 또는 `$effect`로 `filterTags` 변화 감지:
```javascript
$effect(() => {
    if (selectedTagId && !filterTags.some(t => t.id === selectedTagId)) {
        selectedTagId = null;
    }
});
```

## 테스트 케이스

- [ ] AI 태그 기사 1개만 있을 때 삭제 → selectedTagId nil 초기화, 전체 목록 표시
- [ ] AI 태그 기사 2개 중 1개 삭제 → selectedTagId 유지, 나머지 1개 표시
- [ ] 오답 탭 선택 중 연결된 즐겨찾기 삭제 → 동일 동작 확인

## 메모

M4에서 iOS/Web 모두 미처리. 다음 버그 수정 마일스톤에서 iOS + Web 동시 수정 권장.
