import SwiftUI

/// MVP7 M4: QuizView — 퀴즈 3문제 순차 진행 화면.
///
/// ArticleDetailView에서 .sheet 형태로 표시.
/// 닫히면 QuizFeature.reset() 호출로 상태 초기화.
struct QuizView: View {
    let questions: [QuizQuestion]
    let onClose: () -> Void

    @State private var currentIndex = 0
    @State private var selectedIndex: Int? = nil
    @State private var score = 0
    @State private var finished = false

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

            // 해설 (선택 후)
            if let selected = selectedIndex {
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
        let isAnswered = selectedIndex != nil
        let isAnswer = index == question.answerIndex

        Button {
            guard selectedIndex == nil else { return }
            selectedIndex = index
            if index == question.answerIndex {
                score += 1
            }
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
                isAnswered
                    ? (isAnswer
                        ? Color.green.opacity(0.15)
                        : (isSelected ? Color.red.opacity(0.15) : Color(.systemGray6)))
                    : Color(.systemGray6)
            )
            .foregroundStyle(
                isAnswered
                    ? (isAnswer ? .green : (isSelected ? .red : Color(.label)))
                    : Color(.label)
            )
            .clipShape(RoundedRectangle(cornerRadius: 10))
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .stroke(
                        isAnswered
                            ? (isAnswer ? Color.green : (isSelected ? Color.red : Color(.systemGray4)))
                            : Color(.systemGray4),
                        lineWidth: 1
                    )
            )
        }
        .buttonStyle(.plain)
        .disabled(isAnswered)
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
        } else {
            currentIndex += 1
            selectedIndex = nil
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
