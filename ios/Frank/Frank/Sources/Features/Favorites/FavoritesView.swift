import SwiftUI

/// MVP5 M3: FavoritesView — 스크랩 목록 탭.
/// step-5 L 반영: FavoriteItem.summary/insight → SummarySessionCache에 주입
struct FavoritesView: View {
    let feature: FavoritesFeature
    let summarize: any SummarizePort

    var body: some View {
        NavigationStack {
            content
                .navigationTitle("스크랩")
                .task { await feature.loadFavorites() }
                .overlay(alignment: .bottom) {
                    if let errorMsg = feature.operationError {
                        operationErrorBanner(message: errorMsg)
                    }
                }
        }
    }

    @ViewBuilder
    private var content: some View {
        switch feature.phase {
        case .loading:
            loadingView

        case .failed(let message):
            errorView(message: message)

        case .idle, .done:
            if feature.items.isEmpty && feature.hasLoaded {
                emptyView
            } else {
                itemList
            }
        }
    }

    // MARK: - Loading

    private var loadingView: some View {
        VStack(spacing: 12) {
            ProgressView()
            Text("불러오는 중...")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - Error

    private func errorView(message: String) -> some View {
        VStack(spacing: 12) {
            Image(systemName: "exclamationmark.triangle")
                .font(.largeTitle)
                .foregroundStyle(.orange)
            Text(message)
                .font(.body)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
            Button("다시 시도") {
                Task { await feature.loadFavorites() }
            }
            .buttonStyle(.bordered)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding()
    }

    // MARK: - Empty

    private var emptyView: some View {
        VStack(spacing: 12) {
            Image(systemName: "star")
                .font(.system(size: 48))
                .foregroundStyle(.yellow)
            Text("즐겨찾기한 기사가 없습니다")
                .font(.headline)
            Text("피드에서 기사를 읽고 즐겨찾기를 추가해보세요.")
                .font(.caption)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding()
    }

    // MARK: - List

    private var itemList: some View {
        List(feature.items) { item in
            NavigationLink(value: item) {
                FavoriteRowView(item: item)
            }
            .swipeActions(edge: .trailing) {
                Button(role: .destructive) {
                    Task { await feature.removeFavorite(url: item.url) }
                } label: {
                    Label("삭제", systemImage: "trash")
                }
            }
        }
        .listStyle(.plain)
        .navigationDestination(for: FavoriteItem.self) { item in
            favoriteDetail(item: item)
        }
    }

    @ViewBuilder
    private func favoriteDetail(item: FavoriteItem) -> some View {
        if let url = URL(string: item.url) {
            let feedItem = FeedItem(
                title: item.title,
                url: url,
                source: item.source,
                publishedAt: item.publishedAt,
                tagId: item.tagId,
                snippet: item.snippet
            )
            let _ = injectSummaryCache(item: item, url: url.absoluteString)
            ArticleDetailView(
                feedItem: feedItem,
                summarize: summarize,
                favoritesFeature: feature
            )
        }
    }

    // MARK: - Operation Error Banner

    /// add/remove 변이 실패 시 화면 하단에 표시하는 인라인 에러 배너.
    /// 탭하면 dismiss.
    private func operationErrorBanner(message: String) -> some View {
        Text(message)
            .font(.footnote)
            .foregroundStyle(.white)
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
            .background(Color.red.opacity(0.85))
            .clipShape(RoundedRectangle(cornerRadius: 8))
            .padding(.bottom, 16)
            .onTapGesture { feature.clearOperationError() }
            .transition(.move(edge: .bottom).combined(with: .opacity))
    }

    // MARK: - Summary Cache Injection

    /// step-5 L: 저장된 요약 → SummarySessionCache 주입 (상세 진입 시 즉시 표시)
    @discardableResult
    private func injectSummaryCache(item: FavoriteItem, url: String) -> Bool {
        if let summary = item.summary, let insight = item.insight {
            SummarySessionCache.shared.set(url, SummaryResult(summary: summary, insight: insight))
            return true
        }
        return false
    }
}

/// 즐겨찾기 목록 행 뷰.
struct FavoriteRowView: View {
    let item: FavoriteItem

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(item.title)
                .font(.body)
                .fontWeight(.medium)
                .lineLimit(2)

            HStack(spacing: 4) {
                Text(item.source)
                    .font(.caption)
                    .foregroundStyle(.secondary)

                if let createdAt = item.createdAt {
                    Text("·")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Text(relativeDate(createdAt))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                if item.summary != nil {
                    Spacer()
                    Image(systemName: "text.quote")
                        .font(.caption)
                        .foregroundStyle(.indigo)
                }
            }
        }
        .padding(.vertical, 4)
    }

    private func relativeDate(_ date: Date) -> String {
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .short
        return formatter.localizedString(for: date, relativeTo: Date())
    }
}
