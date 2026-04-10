import SwiftUI

/// MVP5 M1: FeedView — ephemeral 피드 표시.
/// collectAndRefresh 버튼 제거. pull-to-refresh = API 재호출.
/// NavigationLink value: String (FeedItem.id = url absoluteString 기반).
struct FeedView: View {
    let feature: FeedFeature
    let summarize: any SummarizePort
    let favoritesFeature: FavoritesFeature
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

                errorBanner

                mainContent
            }
            .navigationTitle("Frank")
            .toolbar {
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
            .navigationDestination(for: String.self) { urlString in
                if let item = feature.articles.first(where: { $0.id == urlString }) {
                    ArticleDetailView(feedItem: item, summarize: summarize, favoritesFeature: favoritesFeature)
                }
            }
            .task {
                await feature.send(.loadInitial)
            }
            .onReceive(NotificationCenter.default.publisher(for: UIApplication.willEnterForegroundNotification)) { _ in
                Task { await feature.send(.refresh) }
            }
        }
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
            ForEach(feature.articles) { item in
                NavigationLink(value: item.id) {
                    ArticleCardView(article: item)
                }
                .buttonStyle(.plain)
            }
        }
        .listStyle(.plain)
        .refreshable {
            await feature.send(.refresh)
        }
    }
}
