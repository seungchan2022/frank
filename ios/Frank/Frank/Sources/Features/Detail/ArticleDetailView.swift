import SwiftUI

/// MVP5 M1: ArticleDetailView — FeedItem 직접 표시.
/// 비동기 로딩 불필요 (ephemeral 피드 아이템을 직접 수신).
struct ArticleDetailView: View {
    let feedItem: FeedItem

    @Environment(\.openURL) private var openURL

    var body: some View {
        articleContent(feedItem)
            .navigationBarTitleDisplayMode(.inline)
    }
}

// MARK: - Article Content

extension ArticleDetailView {
    @ViewBuilder
    private func articleContent(_ item: FeedItem) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // 제목
                Text(item.title)
                    .font(.title2)
                    .fontWeight(.bold)

                // 출처 + 날짜
                HStack(spacing: 4) {
                    Text(item.source)
                    Text("\u{00B7}")
                    Text(ArticleCardView.relativeTimeText(item.publishedAt))
                }
                .font(.caption)
                .foregroundStyle(.secondary)

                Divider()

                // 원문 리드 섹션
                if let snippet = item.snippet {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("원문 리드")
                            .font(.subheadline)
                            .fontWeight(.bold)
                            .foregroundStyle(.secondary)

                        Text(snippet)
                            .font(.body)
                    }

                    Divider()
                }

                // 원문 보기 버튼
                Button {
                    openURL(item.url)
                } label: {
                    HStack {
                        Image(systemName: "safari")
                        Text("원문 보기")
                    }
                    .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
            }
            .padding(.horizontal, 20)
            .padding(.vertical, 16)
        }
    }
}
