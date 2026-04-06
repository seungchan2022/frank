import SwiftUI

struct EmptyStateView: View {
    var icon: String = "newspaper"
    var message: String = "이 키워드의 뉴스가 아직 없습니다"

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: icon)
                .font(.system(size: 48))
                .foregroundStyle(.secondary)

            Text(message)
                .font(.body)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding()
        .accessibilityElement(children: .combine)
        .accessibilityLabel(message)
    }
}

// MARK: - Preview

#Preview("Default") {
    EmptyStateView()
}

#Preview("Custom message") {
    EmptyStateView(
        icon: "magnifyingglass",
        message: "검색 결과가 없습니다"
    )
}
