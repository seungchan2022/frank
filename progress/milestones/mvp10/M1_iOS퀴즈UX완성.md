# M1: iOS 퀴즈 UX 완성

> 프로젝트: Frank MVP10
> 상태: 대기
> 예상 기간: 1~2일
> 의존성: 없음 (서버 API는 이미 quiz_completed 반환 — 단, 서버 레이어 완료 후 진행 권장)
> 실행 순서: 3단계 — 서버/DB(1단계) + 웹(2단계) 완료 후 iOS 진행

## 목표

iOS 퀴즈 완료 화면에 웹과 동등한 UX(다시 풀기 + 오답 보기)를 제공하고,
즐겨찾기 목록에서 퀴즈 완료 배지가 정상 표시되도록 수정한다.

## 성공 기준 (Definition of Done)

- [ ] QuizView `finishedView`에 "다시 풀기" 버튼 추가 — 탭 시 상태 리셋 후 동일 문제 세트 재시작
- [ ] QuizView `finishedView`에 "오답 보기" 버튼 추가 — 오답 있을 때만 활성화, 탭 시 시트 표시 (웹과 동일)
- [ ] FavoritesView에서 `quizCompleted` 배지가 실제 DB 기준으로 표시됨
- [ ] 서버 `GET /api/me/favorites` 응답의 `quiz_completed` 필드 정상 반환 확인
- [ ] iOS Swift Testing 관련 테스트 통과
- [ ] 커버리지 90% 유지

## 아이템

| # | 아이템 | 레이어 | 실행 단계 | 유형 | 상태 |
|---|--------|--------|----------|------|------|
| 1 | iOS QuizView 완료 화면 — 다시 풀기 + 오답 보기 버튼 추가 (웹과 동일) | iOS | 3단계 | feature | 대기 |
| 2 | iOS 퀴즈 배지 미표시 — FavoritesFeature.markQuizCompleted(url:) 추가 | iOS | 3단계 | feature | 대기 |

> 서버/DB(1단계) + 웹(2단계) 완료 후 마지막에 진행.

---

## 상세 분석

### ST-1: 퀴즈 완료 화면 버튼 추가

**파일**: `ios/Frank/Frank/Sources/Features/Quiz/QuizView.swift`

**현재 상태 (`finishedView`)**: "닫기" 버튼 1개만 존재.
MVP9 M2에서 인라인 오답 카드(`wrongAccumulator.items` 기반)가 구현되어 있으나,
"다시 풀기"와 "오답 보기(시트)" 버튼은 없음.

**수정 방향 — 웹과 동일하게 구현**:

웹 퀴즈 완료 화면과 동일하게 두 버튼을 추가한다.

- **"다시 풀기"**: 탭 시 QuizView 내 상태 변수 전체 원자적 리셋 후 동일 문제 세트 재시작
- **"오답 보기"**: 탭 시 시트(Sheet) 형태로 오답 목록 표시 — 오답이 없을 때는 비활성화

**상태 리셋 완결성 체크리스트** (원자적 리셋 필수 — 하나라도 빠지면 이전 세션 데이터 잔류):
- `currentIndex = 0`
- `selectedIndex = nil`
- `confirmed = false`
- `score = 0`
- `finished = false`
- `quizCompletedMarked = false`
- `wrongAccumulator.reset()`

**콜백 구조**:
- `QuizView`에 `onRetry: (() -> Void)?` 콜백 추가
- `onRetry` 탭 시 상태 리셋 후 콜백 호출 → 호출 측에서 새 퀴즈 생성 트리거
- `onRetry == nil`이면 "다시 풀기" 버튼 자체를 숨김

**호출 측 수정** (`ArticleDetailView.swift`):
- `QuizView` 생성 시 `onRetry` 클로저 추가
- `onRetry` 내에서 `quizFeature.generateQuiz(url:title:)` 재호출

---

### ST-2: iOS 퀴즈 배지 미표시 수정

**증상**: FavoritesView에서 퀴즈를 완료해도 배지(checkmark.circle.fill)가 나타나지 않음.

**실제 버그 경로**:

```
QuizView.nextQuestion() → onQuizCompleted?() 호출
→ ArticleDetailView: quizFeature.markQuizCompleted()
→ QuizFeature: FavoritesPort 직접 보유 → 서버 DB만 업데이트
→ FavoritesFeature.items 배열은 갱신되지 않음  ← 버그 위치
→ FavoritesFeature.isQuizCompleted(url) → false 반환 (stale cache)
```

핵심: `QuizFeature`와 `FavoritesFeature`가 각각 별도의 `FavoritesPort`를 사용하여
서버는 업데이트되지만 `FavoritesFeature.items`(UI 상태)가 갱신되지 않음.

**수정 방향 — A안 확정**:

`FavoritesFeature`에 `markQuizCompleted(url:)` 메서드를 추가한다.
`addFavorite` / `removeFavorite`와 동일한 패턴 — Port 호출 + 로컬 상태 동기화.

- `ArticleDetailView.onQuizCompleted` 콜백: `quizFeature.markQuizCompleted()` → `favoritesFeature.markQuizCompleted(url:)` 로 교체
- `QuizFeature.markQuizCompleted()`: 서버 호출 제거, 빈 상태로 유지 또는 삭제 (서버 호출 책임을 FavoritesFeature로 이관)

**보완 조건**:
1. **상태 동기화 필수**: 서버 호출 성공 후 `items[idx].quizCompleted = true` 로컬 갱신 (`addFavorite` 패턴 동일)
2. **TDD 순서 강제**: 실패 테스트 먼저 작성 → 구현 → 통과 확인

**구현 의사코드**:

```
FavoritesFeature.markQuizCompleted(url:)
  1. operationError = nil
  2. favorites.markQuizCompleted(url:) 호출 (await, throws)
  3. 성공 시: items에서 해당 url 찾아 quizCompleted = true 갱신
  4. 실패 시: operationError 설정
```

---

## 테스트 계획

| 테스트 | 파일 위치 | 검증 내용 |
|--------|----------|----------|
| QuizView finishedView — 다시 풀기 버튼 탭 시 상태 7개 전부 리셋 | `FrankTests/QuizTests/` | 상태 변수 7개 모두 초기값 확인 |
| QuizView finishedView — onRetry=nil 시 다시 풀기 버튼 미표시 | `FrankTests/QuizTests/` | nil이면 버튼 숨김 |
| QuizView finishedView — 오답 보기 버튼: 오답 있을 때만 활성화 | `FrankTests/QuizTests/` | wrongAccumulator.items.isEmpty 기반 활성/비활성 |
| FavoritesFeature.markQuizCompleted 성공 시 items 갱신 | `FrankTests/FavoritesTests/` | isQuizCompleted(url) → true |
| FavoritesFeature.markQuizCompleted 실패 시 operationError 설정 | `FrankTests/FavoritesTests/` | operationError != nil |

---

## 리스크

| 리스크 | 영향 | 대응 |
|--------|------|------|
| 상태 리셋 항목 누락 시 이전 세션 데이터 잔류 | H | 체크리스트 7개 항목 모두 확인 |
| FavoriteItem struct vs class — 불변 복사 패턴 필요 | M | FavoriteItem.swift 타입 확인 후 적절한 갱신 방식 선택 |
| onRetry 없는 맥락에서 다시 풀기 미표시 | L | onRetry가 nil일 때 버튼 자체를 숨김 처리로 해결 |
| QuizFeature 서버 호출 제거 시 다른 호출 경로 영향 | M | QuizFeature.markQuizCompleted() 전체 사용처 확인 후 이관 |
