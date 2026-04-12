import SwiftUI

/// MVP7 M4: QuizView — 퀴즈 3문제 순차 진행 화면.
///
/// ArticleDetailView에서 .sheet 형태로 표시.
/// 닫히면 QuizFeature.reset() 호출로 상태 초기화.
struct QuizView: View {
    let questions: [QuizQuestion]
    let onClose: () -> Void
    /// MVP8 M3: 오답 발생 시 호출 (fire-and-forget)
    var onWrongAnswer: ((QuizQuestion, Int) -> Void)? = nil
    /// MVP8 M3: 퀴즈 완료 시 호출 (중복 방지는 내부에서 처리)
    var onQuizCompleted: (() -> Void)? = nil

    @State private var currentIndex = 0
    @State private var selectedIndex: Int? = nil
    @State private var confirmed = false
    @State private var score = 0
    @State private var finished = false
    /// 퀴즈 완료 마킹 중복 방지 플래그
    @State private var quizCompletedMarked = false

    private var currentQuestion: QuizQuestion? {
        guard currentIndex < questions.count else { return nil }
        return questions[currentIndex]
    }

    private var isCorrect: Bool {
        guard let selected = selectedIndex,
              let question = currentQuestion else { return false }
        return selected == question.answerIndex
    }

    var body: some View {
        NavigationStack {
            Group {
                if finished {
                    finishedView
                } else if let question = currentQuestion {
                    questionView(question)
                }
            }
            .navigationTitle("퀴즈")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("닫기") {
                        onClose()
                    }
                }
            }
        }
    }

    // MARK: - Question View

    @ViewBuilder
    private func questionView(_ question: QuizQuestion) -> some View {
        VStack(alignment: .leading, spacing: 20) {
            // 진행 표시
            VStack(spacing: 4) {
                HStack {
                    Text("문제 \(currentIndex + 1) / \(questions.count)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Spacer()
                }
                ProgressView(value: Double(currentIndex + 1), total: Double(questions.count))
                    .tint(.indigo)
            }

            // 질문
            Text(question.question)
                .font(.headline)
                .fixedSize(horizontal: false, vertical: true)

            // 보기
            VStack(spacing: 10) {
                ForEach(Array(question.options.enumerated()), id: \.offset) { index, option in
                    optionButton(option: option, index: index, question: question)
                }
            }

            // 확인 버튼 (선택 후, 확인 전)
            if selectedIndex != nil && !confirmed {
                Button("확인") {
                    guard let selected = selectedIndex else { return }
                    if selected == question.answerIndex {
                        score += 1
                    } else {
                        // 오답 시 fire-and-forget 저장
                        onWrongAnswer?(question, selected)
                    }
                    confirmed = true
                }
                .buttonStyle(.borderedProminent)
                .tint(.indigo)
                .frame(maxWidth: .infinity)
            }

            // 해설 + 다음 버튼 (확인 후)
            if confirmed, let selected = selectedIndex {
                explanationView(question: question, selectedIndex: selected)

                Button(currentIndex + 1 >= questions.count ? "결과 보기" : "다음 문제") {
                    nextQuestion()
                }
                .buttonStyle(.borderedProminent)
                .tint(.indigo)
                .frame(maxWidth: .infinity)
            }

            Spacer()
        }
        .padding()
    }

    @ViewBuilder
    private func optionButton(option: String, index: Int, question: QuizQuestion) -> some View {
        let isSelected = selectedIndex == index
        let isAnswer = index == question.answerIndex

        Button {
            guard !confirmed else { return } // 확인 후에는 변경 불가
            selectedIndex = index
        } label: {
            HStack {
                Text("\(String(UnicodeScalar(65 + index)!)).")
                    .font(.subheadline.bold())
                    .frame(width: 20)
                Text(option)
                    .font(.subheadline)
                    .fixedSize(horizontal: false, vertical: true)
                Spacer()
            }
            .padding()
            .background(
                confirmed
                    ? (isAnswer
                        ? Color.green.opacity(0.15)
                        : (isSelected ? Color.red.opacity(0.15) : Color(.systemGray6)))
                    : (isSelected ? Color.indigo.opacity(0.12) : Color(.systemGray6))
            )
            .foregroundStyle(
                confirmed
                    ? (isAnswer ? .green : (isSelected ? .red : Color(.label)))
                    : (isSelected ? Color.indigo : Color(.label))
            )
            .clipShape(RoundedRectangle(cornerRadius: 10))
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .stroke(
                        confirmed
                            ? (isAnswer ? Color.green : (isSelected ? Color.red : Color(.systemGray4)))
                            : (isSelected ? Color.indigo : Color(.systemGray4)),
                        lineWidth: 1
                    )
            )
        }
        .buttonStyle(.plain)
        .disabled(confirmed)
    }

    @ViewBuilder
    private func explanationView(question: QuizQuestion, selectedIndex: Int) -> some View {
        let correct = selectedIndex == question.answerIndex
        HStack(alignment: .top, spacing: 8) {
            Text(correct ? "✓ 정답!" : "✗ 오답")
                .font(.subheadline.bold())
                .foregroundStyle(correct ? .green : .red)
            Text(question.explanation)
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
        }
        .padding()
        .background(correct ? Color.green.opacity(0.08) : Color.red.opacity(0.08))
        .clipShape(RoundedRectangle(cornerRadius: 10))
    }

    // MARK: - Finished View

    @ViewBuilder
    private var finishedView: some View {
        VStack(spacing: 24) {
            Spacer()

            Text(score == questions.count ? "🎉" : score >= questions.count / 2 ? "👍" : "📚")
                .font(.system(size: 60))

            Text("퀴즈 완료!")
                .font(.title2.bold())

            Text("\(score) / \(questions.count)")
                .font(.system(size: 48, weight: .bold))
                .foregroundStyle(.indigo)

            Text(
                score == questions.count
                    ? "완벽합니다! 기사 내용을 완전히 이해했습니다."
                    : score >= questions.count / 2
                        ? "잘 이해했습니다. 틀린 문제를 다시 확인해보세요."
                        : "기사를 다시 읽고 복습해보세요."
            )
            .font(.subheadline)
            .foregroundStyle(.secondary)
            .multilineTextAlignment(.center)
            .padding(.horizontal)

            Spacer()

            Button("닫기") {
                onClose()
            }
            .buttonStyle(.borderedProminent)
            .tint(.indigo)
            .frame(maxWidth: .infinity)
            .padding(.horizontal)
        }
        .padding()
    }

    // MARK: - Actions

    private func nextQuestion() {
        if currentIndex + 1 >= questions.count {
            finished = true
            // 퀴즈 완료 시 1회만 마킹
            if !quizCompletedMarked {
                quizCompletedMarked = true
                onQuizCompleted?()
            }
        } else {
            currentIndex += 1
            selectedIndex = nil
            confirmed = false
        }
    }
}

#if DEBUG
#Preview {
    QuizView(
        questions: MockFixtures.quizQuestions,
        onClose: {}
    )
}
#endif
