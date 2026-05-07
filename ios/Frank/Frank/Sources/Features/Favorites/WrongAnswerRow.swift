import SwiftUI

/// MVP8 M3: WrongAnswerRow — 오답 노트 목록 행 뷰.
/// 기사 제목 / 문제 / 내 답 (빨간) / 정답 (초록) / 해설 표시.
/// MVP16 M4 D2: 원문 보기 버튼 추가 — SFSafariViewController(인앱 웹뷰).
struct WrongAnswerRow: View {
    let item: WrongAnswer

    @State private var showSafari = false

    /// http/https scheme만 허용 — SFSafariViewController는 http/https 외 scheme 지원 안 함
    private var articleURL: URL? {
        guard let url = URL(string: item.articleUrl),
              let scheme = url.scheme?.lowercased(),
              scheme == "http" || scheme == "https" else { return nil }
        return url
    }

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

            // 원문 보기 버튼 (MVP16 M4 D2)
            // articleURL은 http/https scheme 검증을 거침 — nil이면 버튼 미표시
            if articleURL != nil {
                Button {
                    showSafari = true
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: "safari")
                        Text("원문 보기")
                    }
                    .font(.caption)
                    .foregroundStyle(Color.accentColor)
                }
                .buttonStyle(.plain)
                .accessibilityLabel("원문 기사 열기")
            }
        }
        .padding(.vertical, 8)
        // sheet는 뷰 계층 상위에 위치해야 레이어 충돌 방지
        .sheet(isPresented: $showSafari) {
            if let url = articleURL {
                SafariView(url: url)
                    .ignoresSafeArea()
            }
        }
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
