import SwiftUI

/// MVP8 M3: WrongAnswerRow — 오답 노트 목록 행 뷰.
/// 기사 제목 / 문제 / 내 답 (빨간) / 정답 (초록) / 해설 표시.
struct WrongAnswerRow: View {
    let item: WrongAnswer

    private var myAnswer: String {
        guard item.userIndex < item.options.count else { return "-" }
        return item.options[item.userIndex]
    }

    private var correctAnswer: String {
        guard item.correctIndex < item.options.count else { return "-" }
        return item.options[item.correctIndex]
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            // 기사 제목
            Text(item.articleTitle)
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(1)

            // 질문
            Text(item.question)
                .font(.subheadline)
                .fontWeight(.semibold)
                .fixedSize(horizontal: false, vertical: true)

            // 내 답 / 정답
            HStack(spacing: 8) {
                answerBadge(label: "내 답", text: myAnswer, color: .red)
                answerBadge(label: "정답", text: correctAnswer, color: .green)
            }

            // 해설
            if let explanation = item.explanation, !explanation.isEmpty {
                Text(explanation)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .padding(.vertical, 8)
    }

    @ViewBuilder
    private func answerBadge(label: String, text: String, color: Color) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(label)
                .font(.caption2)
                .foregroundStyle(color)
                .fontWeight(.semibold)
            Text(text)
                .font(.caption)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(color.opacity(0.1))
                .foregroundStyle(color)
                .clipShape(RoundedRectangle(cornerRadius: 6))
        }
    }
}
