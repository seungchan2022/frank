# M4: iOS 즐겨찾기·오답 태그 필터 (BUG-003)

> 프로젝트: Frank MVP11
> 상태: 대기
> 예상 기간: 1~2일
> 의존성: 없음 (M1과 독립적)

## 목표

즐겨찾기(스크랩)·오답 화면에 피드와 동일한 태그 칩 필터 UI를 추가하여,
콘텐츠 누적 시에도 원하는 태그로 빠르게 탐색할 수 있도록 한다.

## 성공 기준 (Definition of Done)

- [ ] `FavoritesView` 기사 탭에 태그 칩 필터 UI 표시 (피드와 동일 컴포넌트 재사용)
- [ ] `WrongAnswersView`(오답 노트 탭)에 태그 칩 필터 UI 표시
- [ ] 태그 선택 시 해당 태그 기사/오답만 표시 (클라이언트 사이드 필터링)
- [ ] "전체" 칩 선택 시 필터 해제 — 전체 목록 복원
- [ ] 기존 즐겨찾기·오답 iOS 테스트 전체 통과 (회귀 없음)

## 아이템

| # | 아이템 | 유형 | 실행 스킬 | 상태 |
|---|--------|------|----------|------|
| 1 | FavoritesFeature에 태그 필터 상태 추가 | feature | /workflow | 대기 |
| 2 | FavoritesView 기사 탭에 태그 칩 필터 UI 추가 | feature | /workflow | 대기 |
| 3 | WrongAnswersFeature에 태그 필터 상태 추가 | feature | /workflow | 대기 |
| 4 | WrongAnswersView(오답 탭)에 태그 칩 필터 UI 추가 | feature | /workflow | 대기 |

## 구현 방향

### 필터링 방식: 클라이언트 사이드

- API는 이미 태그 데이터를 반환 중 → 서버 변경 불필요
- `FavoritesFeature` / `WrongAnswersFeature`에 `selectedTag: String?` 상태 추가
- 피드의 태그 칩 컴포넌트(`TagChipFilterView` 또는 유사 컴포넌트) 재사용

### 태그 목록 수집 방법

- 현재 로드된 기사/오답 목록에서 태그를 추출 → 중복 제거 → 칩으로 표시
- "전체" 칩은 항상 첫 번째, 나머지는 알파벳/가나다 정렬

### UI 레이아웃

```
[스크랩]
─────────────────
[전체] [AI] [iOS] [Swift] ...   ← 태그 칩 필터 (스크롤 가능 HStack)
─────────────────
기사 목록 (필터 적용)
```

## 리스크

| 리스크 | 영향(H/M/L) | 대응 |
|--------|------------|------|
| 태그 없는 기사(tag = nil)의 필터 처리 | L | "전체" 칩에만 포함, 특정 태그 선택 시 제외 |
| 태그 칩이 너무 많을 때 레이아웃 오버플로우 | L | `ScrollView(.horizontal)` + 고정 높이 |

---

## KPI (M4)

| 지표 | 측정 방법 | 목표 | 게이트 | 기준선 |
|---|---|---|---|---|
| M2 DoD 테스트 통과 | `xcodebuild test -workspace Frank.xcworkspace -scheme Frank -destination 'platform=iOS Simulator,name=iPhone 17 Pro'` | 전체 통과 | Hard | — |
| BUG-003 재현 0건 | 시뮬레이터에서 즐겨찾기·오답 탭 태그 필터 동작 수동 확인 | 0건 | Hard | 필터 없음 |
| 태그 필터 선택/해제 동작 | 시뮬레이터 수동 UI 확인 | 정상 동작 | Hard | — |
