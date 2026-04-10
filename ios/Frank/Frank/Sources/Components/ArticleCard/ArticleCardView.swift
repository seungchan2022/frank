import SwiftUI

/// MVP6 M1: ArticleCardView — 썸네일 + 텍스트 HStack 레이아웃.
/// 왼쪽 72×72 썸네일(AsyncImage 또는 플레이스홀더) + 오른쪽 제목·source·발행일.
struct ArticleCardView: View {
    let article: Article

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            // 썸네일 영역 (72×72)
            thumbnailView

            // 텍스트 영역
            VStack(alignment: .leading, spacing: 6) {
                Text(article.title)
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .foregroundStyle(.primary)
                    .multilineTextAlignment(.leading)
                    .lineLimit(2)

                HStack(spacing: 4) {
                    Text(article.source)
                    Text("\u{00B7}")
                    Text(Self.relativeTimeText(article.publishedAt))
                }
                .font(.caption)
                .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(.vertical, 8)
        .accessibilityElement(children: .combine)
        .accessibilityLabel(article.title)
    }

    // MARK: - Thumbnail

    @ViewBuilder
    private var thumbnailView: some View {
        if let imageUrl = article.imageUrl {
            AsyncImage(url: imageUrl) { phase in
                switch phase {
                case .success(let image):
                    image
                        .resizable()
                        .scaledToFill()
                        .frame(width: 72, height: 72)
                        .clipShape(RoundedRectangle(cornerRadius: 8))
                default:
                    // loading / failure → 동일 플레이스홀더 (layout shift 방지)
                    thumbnailPlaceholder
                }
            }
            .frame(width: 72, height: 72)
        } else {
            thumbnailPlaceholder
        }
    }

    private var thumbnailPlaceholder: some View {
        RoundedRectangle(cornerRadius: 8)
            .fill(Color(.systemGray5))
            .frame(width: 72, height: 72)
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
            title: "Breaking news article",
            url: URL(string: "https://example.com")!,
            source: "Reuters",
            publishedAt: Date().addingTimeInterval(-300),
            tagId: UUID()
        ))
    }
    .listStyle(.plain)
}
