import SwiftUI

struct ArticleDetailView: View {
    let feature: ArticleDetailFeature

    @Environment(\.openURL) private var openURL

    var body: some View {
        Group {
            if feature.isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let errorMessage = feature.errorMessage {
                errorView(errorMessage)
            } else if let article = feature.article {
                articleContent(article)
            }
        }
        .navigationBarTitleDisplayMode(.inline)
        .task {
            await feature.send(.loadArticle)
        }
    }
}

// MARK: - Article Content

extension ArticleDetailView {
    @ViewBuilder
    private func articleContent(_ article: Article) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // 제목
                Text(article.titleKo ?? article.title)
                    .font(.title2)
                    .fontWeight(.bold)

                // 출처 + 날짜
                HStack(spacing: 4) {
                    Text(article.source)
                    Text("\u{00B7}")
                    Text(ArticleCardView.relativeTimeText(article.publishedAt))
                }
                .font(.caption)
                .foregroundStyle(.secondary)

                Divider()

                // AI 요약 섹션
                VStack(alignment: .leading, spacing: 8) {
                    Text("AI 요약")
                        .font(.subheadline)
                        .fontWeight(.bold)
                        .foregroundStyle(.secondary)

                    if let summary = article.summary {
                        Text(summary)
                            .font(.body)
                    } else {
                        Text("요약 중...")
                            .font(.body)
                            .italic()
                            .foregroundStyle(.secondary)
                    }
                }

                // 핵심 인사이트 섹션
                if let insight = article.insight {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("핵심 인사이트")
                            .font(.subheadline)
                            .fontWeight(.bold)
                            .foregroundStyle(.secondary)

                        Text(insight)
                            .font(.body)
                            .foregroundStyle(.tint)
                    }
                }

                Divider()

                // 원문 보기 버튼
                Button {
                    openURL(article.url)
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

// MARK: - Error View

extension ArticleDetailView {
    @ViewBuilder
    private func errorView(_ message: String) -> some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle")
                .font(.largeTitle)
                .foregroundStyle(.secondary)

            Text(message)
                .font(.body)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)

            Button("다시 시도") {
                Task {
                    await feature.send(.loadArticle)
                }
            }
            .buttonStyle(.bordered)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
