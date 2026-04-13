import SwiftUI

/// 로딩 중 표시되는 텍스트 — 지정된 delay 후 `afterText`로 전환.
///
/// 사용 예:
/// ```swift
/// LoadingTextView(initial: "요약 중…", after: "마무리 중…", delay: 8)
/// LoadingTextView(initial: "퀴즈 생성 중…", after: "마무리 중…", delay: 8)
/// ```
struct LoadingTextView: View {
    let initial: String
    let after: String
    let delay: TimeInterval

    @State private var text: String
    @State private var isAfter = false

    init(initial: String, after: String, delay: TimeInterval = 8) {
        self.initial = initial
        self.after = after
        self.delay = delay
        self._text = State(initialValue: initial)
    }

    var body: some View {
        HStack {
            ProgressView()
                .tint(isAfter ? .orange : .indigo)
                .padding(.trailing, 4)
            Text(text)
                .foregroundStyle(isAfter ? AnyShapeStyle(.orange) : AnyShapeStyle(.indigo))
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 10)
        .task {
            text = initial
            isAfter = false
            try? await Task.sleep(for: .seconds(delay))
            text = after
            isAfter = true
        }
    }
}
