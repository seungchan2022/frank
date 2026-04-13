# M2: 퀴즈 UX 개선

> 프로젝트: Frank MVP9
> 상태: 완료
> 예상 기간: 1주
> 의존성: 없음 (서버 변경 없음 — 클라이언트 전용)

---

## 목표

퀴즈 완료 후 학습 루프를 완성한다. 오답을 완료 화면에서 바로 확인하고, 기사 상세에서 재도전과 오답 복습이 쉬워진다.

---

## 성공 기준 (Definition of Done)

- [x] 퀴즈 완료 화면: 이번 세션 오답 목록 인라인 표시 (웹·iOS)
- [x] 기사 상세 퀴즈 버튼: 완료 전 "퀴즈 풀기" / 완료 후 "다시 풀기" + "오답 보기" (웹·iOS)
- [x] "오답 보기" 클릭 시 시트로 오답 목록 표시 (웹·iOS)
- [x] 서버 변경 없음 — 기존 엔드포인트·DB 그대로
- [x] 기존 테스트 전부 통과 + 신규 테스트 추가 (웹 216개, iOS 211개)

---

## 아이템

| # | 아이템 | 유형 | 실행 스킬 | 상태 |
|---|--------|------|----------|------|
| 1 | 퀴즈 완료 화면 오답 인라인 표시 (웹+iOS) | feature | /workflow | 완료 |
| 2 | 기사 상세 퀴즈 버튼 재설계 (웹+iOS) | feature | /workflow | 완료 |

---

## 상세 구현 가이드

### 아이템 1: 퀴즈 완료 화면 오답 인라인 표시 (Q1)

**배경**
- 현재: 퀴즈 완료 → 점수(score / questions.length)만 표시. 오답 확인하려면 스크랩 탭 → 오답 노트로 이동 필요.
- 목표: 완료 화면 하단에 "이번 세션 오답" 섹션 인라인 추가.

**핵심 발견 (코드 확인)**
- 현재 오답 발생 시 → 서버 fire-and-forget만 함, **로컬에 누적하지 않음**
- 웹 `QuizModal.svelte`: `confirm()` 함수에서 오답이면 `apiClient.saveWrongAnswer()` 호출 후 끝. 로컬 배열 없음.
- iOS `QuizView.swift`: `onWrongAnswer(question, selected)` 콜백 → `QuizFeature.saveWrongAnswer()` → 서버 저장만 함. 로컬 배열 없음.
- **따라서**: 완료 화면에 표시할 세션 오답을 로컬에 누적하는 상태를 새로 추가해야 함.

**웹 (`QuizModal.svelte`)**

```
변경 위치: web/src/lib/components/QuizModal.svelte

1. 상태 추가
   let sessionWrongAnswers = $state<Array<{
     question: QuizQuestion;
     userIndex: number;
   }>>([]);

2. confirm() 함수 수정 — 오답 발생 시 로컬 누적 추가
   // 기존: apiClient.saveWrongAnswer(...) 만
   // 추가: sessionWrongAnswers = [...sessionWrongAnswers, { question: q, userIndex: uIdx }];

3. 완료 화면(finished = true) 하단에 오답 섹션 추가
   {#if sessionWrongAnswers.length > 0}
     <div>이번 세션 오답 {sessionWrongAnswers.length}개</div>
     {#each sessionWrongAnswers as item}
       문제 / 내 답(빨강) / 정답(초록) / 해설
     {/each}
   {:else}
     완벽! 모두 맞혔어요 (빈 상태)
   {/if}
```

**iOS (`QuizView.swift`)**

```
변경 위치: ios/Frank/.../Features/Quiz/QuizView.swift

핵심 발견:
- WrongAnswerRow는 WrongAnswer(서버 모델 — id, userId, createdAt 포함) 를 받음
- QuizView 로컬 데이터는 QuizQuestion + userIndex만 있음 → 직접 재사용 불가
- 해결: 로컬 전용 구조체 SessionWrongAnswer 정의 후 인라인 렌더링

1. 로컬 구조체 추가 (QuizView 파일 내부)
   private struct SessionWrongAnswer {
     let question: QuizQuestion
     let userIndex: Int
   }

2. 상태 추가 (QuizView 내부)
   @State private var sessionWrongAnswers: [SessionWrongAnswer] = []

3. questionView 확인 버튼 처리 — 오답 발생 시 로컬 누적
   // 기존: onWrongAnswer?(question, selected)
   // 추가: sessionWrongAnswers.append(SessionWrongAnswer(question: question, userIndex: selected))

4. finishedView에 오답 섹션 추가
   if sessionWrongAnswers.isEmpty {
     // "완벽! 모두 맞혔어요" — 기존 메시지 유지
   } else {
     Text("이번 세션 오답 \(sessionWrongAnswers.count)개")
       .font(.headline)
     ForEach(sessionWrongAnswers, id: \.question.question) { item in
       // 인라인 렌더링 (WrongAnswerRow 재사용 안 함)
       // 문제 / 내 답(빨강) / 정답(초록) / 해설
       // WrongAnswerRow 내부의 answerBadge 스타일 참고
     }
   }
```

---

### 아이템 2: 기사 상세 퀴즈 버튼 재설계 (Q2)

**배경**
- 현재: 퀴즈 완료 후에도 "퀴즈 풀기" 버튼만 표시
- 목표: `quiz_completed` 플래그에 따라 버튼 분기

**완료 전 (quiz_completed = false)**
```
[퀴즈 풀기]
```

**완료 후 (quiz_completed = true)**
```
[다시 풀기]  [오답 보기]
```
- "다시 풀기" → 기존 퀴즈 생성 플로우 (generateQuiz 재호출)
- "오답 보기" → 해당 기사의 오답 목록을 **시트(Bottom Sheet)**로 표시
  - `GET /api/quiz/wrong-answers` 응답에서 해당 `article_url` 필터링
  - 이미 스크랩 탭에서 로드된 오답 데이터 활용 (추가 API 호출 최소화)

**웹 (`favorites/+page.svelte` — 기사 상세 섹션)**

```
핵심 발견:
- fav.quizCompleted 플래그 이미 존재 (배지 표시에 활용 중) → 버튼 분기에 바로 사용 가능
- wrongAnswers는 페이지 로컬 상태, 오답 노트 탭을 열 때만 lazy 로딩됨
  → "오답 보기" 클릭 시 아직 로드 안 됐을 수 있음 → 클릭 시점에 fetch 필요
- WrongAnswerCard.svelte 이미 존재 → 오답 시트 내 재사용 가능

변경 내용:
1. 퀴즈 버튼 영역 조건부 분기
   {#if fav.quizCompleted}
     <button onclick={retryQuiz}>다시 풀기</button>
     <button onclick={openWrongAnswerSheet}>오답 보기</button>
   {:else}
     <button onclick={startQuiz}>퀴즈 풀기</button>
   {/if}

2. 오답 시트 상태
   let showWrongAnswerSheet = $state(false);
   let sheetWrongAnswers = $state<WrongAnswer[]>([]);
   let sheetLoading = $state(false);

3. openWrongAnswerSheet 함수
   async function openWrongAnswerSheet() {
     showWrongAnswerSheet = true;
     sheetLoading = true;
     // wrongAnswers가 이미 로드됐으면 재사용, 없으면 fetch
     const all = wrongAnswersLoaded
       ? wrongAnswers
       : await apiClient.listWrongAnswers();
     sheetWrongAnswers = all.filter(wa => wa.articleUrl === fav.url);
     sheetLoading = false;
   }

4. 오답 시트 렌더링
   {#if showWrongAnswerSheet}
     <dialog 또는 모달>
       {#if sheetLoading} 로딩 중...
       {:else if sheetWrongAnswers.length === 0} "이 기사의 오답 기록이 없어요"
       {:else}
         {#each sheetWrongAnswers as wa}
           <WrongAnswerCard {wa} (삭제 버튼 없이 읽기 전용) />
         {/each}
       {/if}
     </dialog>
   {/if}
```

**iOS (`ArticleDetailView.swift` — quizButton)**

```
핵심 발견:
- quizButton이 quizFeature.phase (idle/loading/done)만 보고 있음
- feedItem.quizCompleted 플래그는 FavoriteItem에 있지만 quizButton에서 미사용
- ArticleDetailView에 wrongAnswer: any WrongAnswerPort 이미 주입됨
  → 오답 데이터 fetch에 별도 의존성 추가 불필요
- WrongAnswerRow는 WrongAnswer(서버 모델) 기반 → 오답 시트에서 재사용 가능
  (서버에서 fetch한 데이터이므로 구조 일치)

변경 내용:
1. 상태 추가
   @State private var showWrongAnswerSheet = false
   @State private var sheetWrongAnswers: [WrongAnswer] = []
   @State private var sheetLoading = false

2. quizButton에 quizCompleted 분기 추가
   private var quizButton: some View {
     if feedItem.quizCompleted {
       HStack {
         Button("다시 풀기") {
           Task { await quizFeature.generateQuiz(url: feedItem.url.absoluteString, title: feedItem.title) }
         }
         .buttonStyle(.bordered)
         Button("오답 보기") {
           Task { await loadWrongAnswerSheet() }
         }
         .buttonStyle(.bordered)
       }
     } else {
       // 기존 switch quizFeature.phase 로직 그대로 유지
     }
   }

3. loadWrongAnswerSheet 함수
   private func loadWrongAnswerSheet() async {
     sheetLoading = true
     showWrongAnswerSheet = true
     let all = (try? await wrongAnswer.list()) ?? []
     sheetWrongAnswers = all.filter { $0.articleUrl == feedItem.url.absoluteString }
     sheetLoading = false
   }

4. 오답 시트
   .sheet(isPresented: $showWrongAnswerSheet) {
     NavigationStack {
       if sheetLoading {
         ProgressView()
       } else if sheetWrongAnswers.isEmpty {
         Text("이 기사의 오답 기록이 없어요")
           .foregroundStyle(.secondary)
       } else {
         List(sheetWrongAnswers) { item in
           WrongAnswerRow(item: item)  // 기존 컴포넌트 재사용
         }
       }
     }
     .navigationTitle("오답 보기")
     .navigationBarTitleDisplayMode(.inline)
   }

주의: "다시 풀기" 탭 시 quizCompleted = true 상태이므로 버튼이 계속 분기 유지.
      generateQuiz() 완료 후 phase = .done → QuizView 시트 자동 표시 (기존 동작 유지).
```

---

## 의존성

```
[웹 스트림]
아이템1 (웹 완료 화면 오답 인라인) ← 독립
아이템2 (웹 버튼 재설계) ← 독립

[iOS 스트림]
아이템1 (iOS 완료 화면 오답 인라인) ← 독립
아이템2 (iOS 버튼 재설계) ← 독립

[병렬 가능]
웹 스트림 ∥ iOS 스트림
```

---

## 리스크

| 리스크 | 영향 | 대응 |
|--------|------|------|
| 세션 오답 데이터가 완료 화면에 전달 안 될 수 있음 | M | 퀴즈 진행 중 오답 배열을 상태에 명시적으로 누적. 완료 이벤트 시 배열 snapshot 전달. |
| "오답 보기" 시트에서 해당 기사 오답이 없을 수 있음 | L | 오답 노트에 해당 기사 데이터 없으면 "이 기사의 오답 기록이 없어요" 빈 상태 처리. |
| `quiz_completed` 플래그가 즐겨찾기 안 된 기사에는 없음 | M | 즐겨찾기 안 된 기사 = 퀴즈 완료 배지 없음. 버튼은 항상 "퀴즈 풀기" 단일 표시. |

---

## 워크플로우 참조

```
마일스톤 참조: progress/milestones/frank/M2_퀴즈UX개선.md
브랜치: feature/mvp9-m2-quiz-ux
```

착수 시: `/workflow "MVP9 M2 — 퀴즈 완료 오답 인라인 + 버튼 재설계"`
