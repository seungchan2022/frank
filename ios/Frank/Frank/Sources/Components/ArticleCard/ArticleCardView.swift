import SwiftUI

struct ArticleCardView: View {
    let article: Article

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(article.title)
                .font(.headline)
                .fontWeight(.bold)
                .foregroundStyle(.primary)
                .multilineTextAlignment(.leading)

            HStack(spacing: 4) {
                Text(article.source)
                Text("\u{00B7}")
                Text(Self.relativeTimeText(article.publishedAt ?? article.createdAt))
            }
            .font(.caption)
            .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.vertical, 12)
        .accessibilityElement(children: .combine)
        .accessibilityLabel(article.title)
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

    static func relativeTimeText(_ date: Date?) -> String {
        guard let date else { return "" }
        return relativeFormatter.localizedString(for: date, relativeTo: Date())
    }
}

// MARK: - Preview

#Preview("With published date") {
    List {
        ArticleCardView(article: Article(
            id: UUID(),
            title: "OpenAI releases GPT-5",
            url: URL(string: "https://example.com")!,
            source: "TechCrunch",
            publishedAt: Date().addingTimeInterval(-7200),
            tagId: UUID()
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
