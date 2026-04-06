import SwiftUI

struct ArticleCardView: View {
    let article: Article

    @Environment(\.openURL) private var openURL

    var displayTitle: String {
        article.titleKo ?? article.title
    }

    var body: some View {
        Button {
            openURL(article.url)
        } label: {
            VStack(alignment: .leading, spacing: 8) {
                Text(displayTitle)
                    .font(.headline)
                    .fontWeight(.bold)
                    .foregroundStyle(.primary)
                    .multilineTextAlignment(.leading)

                HStack(spacing: 4) {
                    Text(article.source)
                    Text("\u{00B7}")
                    Text(Self.relativeTimeText(article.publishedAt))
                }
                .font(.caption)
                .foregroundStyle(.secondary)

                if let summary = article.summary {
                    Text(summary)
                        .font(.subheadline)
                        .foregroundStyle(.primary)
                        .lineLimit(2)
                } else {
                    Text("요약 중...")
                        .font(.subheadline)
                        .italic()
                        .foregroundStyle(.secondary)
                }

                if let insight = article.insight {
                    Text(insight)
                        .font(.caption)
                        .foregroundStyle(.tint)
                        .lineLimit(1)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.vertical, 12)
        }
        .buttonStyle(.plain)
        .accessibilityElement(children: .combine)
        .accessibilityLabel(displayTitle)
    }
}

// MARK: - Private Helpers

extension ArticleCardView {
    private static let relativeFormatter: RelativeDateTimeFormatter = {
        let formatter = RelativeDateTimeFormatter()
        formatter.locale = Locale(identifier: "ko_KR")
        formatter.unitsStyle = .short
        return formatter
    }()

    static func relativeTimeText(_ date: Date) -> String {
        relativeFormatter.localizedString(for: date, relativeTo: Date())
    }
}

// MARK: - Preview

#Preview("With all fields") {
    List {
        ArticleCardView(article: Article(
            id: UUID(),
            title: "OpenAI releases GPT-5",
            url: URL(string: "https://example.com")!,
            source: "TechCrunch",
            publishedAt: Date().addingTimeInterval(-7200),
            summary: "OpenAI가 새로운 GPT-5 모델을 출시했습니다. 이전 버전 대비 성능이 크게 향상되었습니다.",
            tagId: UUID(),
            titleKo: "OpenAI, GPT-5 출시",
            insight: "AI 모델 경쟁이 더욱 치열해지고 있음"
        ))
    }
    .listStyle(.plain)
}

#Preview("Without optional fields") {
    List {
        ArticleCardView(article: Article(
            id: UUID(),
            title: "Breaking news article",
            url: URL(string: "https://example.com")!,
            source: "Reuters",
            publishedAt: Date().addingTimeInterval(-300),
            tagId: UUID()
        ))
    }
    .listStyle(.plain)
}
