# M1: iOS 퀴즈 UX 완성

> 프로젝트: Frank MVP10
> 상태: 완료 (260414)
> 예상 기간: 1~2일
> 실제 소요: 1일

## 목표

퀴즈 완료 후 "다시 풀기" + "오답 보기" 버튼이 표시되지 않고,
즐겨찾기 목록에서 퀴즈 완료 배지가 미표시되는 버그를 수정한다.

## 실제 분석 결과 (계획과 달랐던 부분)

**계획**: QuizView finishedView에 "다시 풀기" + "오답 보기" 버튼 추가 필요
**실제**: 버튼은 이미 ArticleDetailView.completedQuizButtons에 구현되어 있었음

버튼이 표시되지 않은 원인은 `isQuizCompleted()` 가 항상 `false` 를 반환하기 때문이었고,
이는 퀴즈 완료 시 서버 DB만 업데이트되고 `FavoritesFeature.items` (UI 상태)가 갱신되지 않아 발생.

## 버그 원인

```
QuizView.nextQuestion() → onQuizCompleted?() 호출
→ ArticleDetailView: quizFeature.markQuizCompleted()
→ QuizFeature: FavoritesPort 직접 보유 → 서버 DB만 업데이트
→ FavoritesFeature.items 갱신 없음  ← 버그 위치
→ FavoritesFeature.isQuizCompleted(url) → false (stale)
→ completedQuizButtons 미표시, 배지 미표시
```

## 실제 수정 내용

| 파일 | 변경 내용 |
|------|----------|
| `FavoritesFeature.swift` | `markQuizCompleted(url:)` 추가 — Port 호출 + items.map 로컬 갱신 |
| `ArticleDetailView.swift` | `onQuizCompleted`에서 `quizFeature.markQuizCompleted()` → `favoritesFeature.markQuizCompleted(url:)` 교체 |
| `FavoritesFeatureTests.swift` | `markQuizCompleted` 관련 테스트 3개 추가 |

## 성공 기준 (Definition of Done)

- [x] 퀴즈 완료 후 ArticleDetailView에서 "다시 풀기" + "오답 보기" 버튼으로 전환됨
- [x] FavoritesView에서 `quizCompleted` 배지 즉시 표시됨
- [x] `FavoritesFeature.markQuizCompleted` 테스트 3개 통과
- [x] 전체 테스트 214개 통과
- [x] 커버리지 기준 유지 (신규 메서드 테스트 3개 추가)

## 리스크 결과

| 리스크 | 결과 |
|--------|------|
| FavoriteItem struct 불변 복사 | items.map + memberwise init으로 해결 |
| QuizFeature 서버 호출 중복 | ArticleDetailView 교체로 해결, QuizFeature 호출 경로 제거 |
