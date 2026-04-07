import SwiftUI

struct FeedView: View {
    let feature: FeedFeature
    let articlePort: any ArticlePort
    var onSettingsTapped: (() -> Void)?

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                TagChipBarView(
                    tags: feature.tags,
                    selectedTagId: feature.selectedTagId,
                    onSelect: { tagId in
                        Task { await feature.send(.selectTag(tagId)) }
                    }
                )
                .padding(.vertical, 8)

                progressBanner

                errorBanner

                mainContent
            }
            .navigationTitle("Frank")
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button {
                        Task { await feature.send(.collectAndSummarize) }
                    } label: {
                        Label("새 뉴스 가져오기", systemImage: "arrow.down.circle")
                    }
                    .disabled(feature.isCollecting || feature.isSummarizing)
                }

                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        onSettingsTapped?()
                    } label: {
                        Image(systemName: "gearshape")
                    }
                    .accessibilityIdentifier("settings_button")
                    .accessibilityLabel("설정")
                }
            }
            .navigationDestination(for: UUID.self) { articleId in
                ArticleDetailView(
                    feature: ArticleDetailFeature(
                        articleId: articleId,
                        articlePort: articlePort
                    )
                )
            }
            .task {
                await feature.send(.loadInitial)
            }
        }
    }

    // MARK: - Progress Banner

    @ViewBuilder
    private var progressBanner: some View {
        if feature.isCollecting {
            bannerRow(text: "뉴스를 수집하고 있어요...")
        } else if feature.isSummarizing {
            bannerRow(text: "AI가 요약하고 있어요...")
        }
    }

    private func bannerRow(text: String) -> some View {
        HStack(spacing: 8) {
            ProgressView()
            Text(text)
                .font(.subheadline)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 10)
        .background(Color(.systemGray6))
    }

    // MARK: - Error Banner

    @ViewBuilder
    private var errorBanner: some View {
        if let errorMessage = feature.errorMessage {
            HStack(spacing: 8) {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(.orange)
                Text(errorMessage)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
            .background(Color.orange.opacity(0.1))
        }
    }

    // MARK: - Main Content

    @ViewBuilder
    private var mainContent: some View {
        if feature.isLoading {
            List {
                ShimmerListView()
            }
            .listStyle(.plain)
        } else if feature.articles.isEmpty {
            EmptyStateView()
        } else {
            articleList
        }
    }

    // MARK: - Article List

    private var articleList: some View {
        List {
            ForEach(feature.articles) { article in
                NavigationLink(value: article.id) {
                    ArticleCardView(article: article)
                }
                .buttonStyle(.plain)
                .onAppear {
                    if article.id == feature.articles.last?.id {
                        Task { await feature.send(.loadMore) }
                    }
                }
            }

            if feature.isLoadingMore {
                HStack {
                    Spacer()
                    ProgressView()
                    Spacer()
                }
                .listRowSeparator(.hidden)
            }
        }
        .listStyle(.plain)
        .refreshable {
            await feature.send(.collectAndSummarize)
        }
    }
}
