import SwiftUI

struct ShimmerCardView: View {
    @State private var isAnimating = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            shimmerRect(width: 260, height: 18)
            shimmerRect(width: 140, height: 12)
            shimmerRect(width: .infinity, height: 14)
            shimmerRect(width: 200, height: 14)
        }
        .padding(.vertical, 12)
        .onAppear { isAnimating = true }
    }

    @ViewBuilder
    private func shimmerRect(width: CGFloat, height: CGFloat) -> some View {
        let maxWidth: CGFloat? = width == .infinity ? nil : width

        RoundedRectangle(cornerRadius: 4)
            .fill(Color.gray.opacity(0.2))
            .frame(maxWidth: maxWidth, minHeight: height, maxHeight: height, alignment: .leading)
            .frame(maxWidth: width == .infinity ? .infinity : nil, alignment: .leading)
            .overlay(
                GeometryReader { geometry in
                    LinearGradient(
                        colors: [
                            Color.clear,
                            Color.white.opacity(0.4),
                            Color.clear,
                        ],
                        startPoint: .leading,
                        endPoint: .trailing
                    )
                    .frame(width: geometry.size.width * 0.4)
                    .offset(x: isAnimating ? geometry.size.width : -geometry.size.width * 0.4)
                    .animation(
                        .linear(duration: 1.5).repeatForever(autoreverses: false),
                        value: isAnimating
                    )
                }
            )
            .clipShape(RoundedRectangle(cornerRadius: 4))
    }
}

struct ShimmerListView: View {
    var count: Int = 5

    var body: some View {
        ForEach(0..<count, id: \.self) { _ in
            ShimmerCardView()
        }
    }
}

// MARK: - Preview

#Preview("Shimmer Card") {
    List {
        ShimmerCardView()
    }
    .listStyle(.plain)
}

#Preview("Shimmer List") {
    List {
        ShimmerListView()
    }
    .listStyle(.plain)
}
