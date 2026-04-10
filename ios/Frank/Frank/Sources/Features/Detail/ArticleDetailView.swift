import SwiftUI

/// MVP5 M3: ArticleDetailView — 온디맨드 요약 + 즐겨찾기 토글 UI.
/// - 요약하기 버튼: idle/loading/done/failed 상태에 따라 UI 전환
/// - 즐겨찾기 버튼: isLiked 상태에 따라 채워진/빈 하트 아이콘
struct ArticleDetailView: View {
    let feedItem: FeedItem
    let favoritesFeature: FavoritesFeature
    private let summarizePort: any SummarizePort
    @State private var feature: ArticleDetailFeature
    @State private var favoriteLoading: Bool = false

    @State private var showSafari = false

    init(feedItem: FeedItem, summarize: any SummarizePort, favoritesFeature: FavoritesFeature) {
        self.feedItem = feedItem
        self.favoritesFeature = favoritesFeature
        self.summarizePort = summarize
        self._feature = State(initialValue: ArticleDetailFeature(feedItem: feedItem, summarize: summarize))
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                headerSection
                Divider()
                snippetSection
                if let errMsg = favoritesFeature.operationError {
                    Text(errMsg)
                        .font(.footnote)
                        .foregroundStyle(.red)
                        .padding(.horizontal, 4)
                        .onTapGesture { favoritesFeature.clearOperationError() }
                }
                actionButtons
                summarySection
            }
            .padding(.horizontal, 20)
            .padding(.vertical, 16)
        }
        .navigationBarTitleDisplayMode(.inline)
        .sheet(isPresented: $showSafari) {
            SafariView(url: feedItem.url)
        }
    }
}

// MARK: - Header

extension ArticleDetailView {
    private var headerSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(feedItem.title)
                .font(.title2)
                .fontWeight(.bold)

            HStack(spacing: 4) {
                Text(feedItem.source)
                Text("\u{00B7}")
                Text(ArticleCardView.relativeTimeText(feedItem.publishedAt))
            }
            .font(.caption)
            .foregroundStyle(.secondary)
        }
    }
}

// MARK: - Snippet

extension ArticleDetailView {
    @ViewBuilder
    private var snippetSection: some View {
        if let snippet = feedItem.snippet {
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
    }
}

// MARK: - Action Buttons

extension ArticleDetailView {
    private var actionButtons: some View {
        VStack(spacing: 10) {
            // 원문 보기
            Button {
                showSafari = true
            } label: {
                HStack {
                    Image(systemName: "safari")
                    Text("원문 보기")
                }
                .frame(maxWidth: .infinity)
            }
            .buttonStyle(.bordered)

            // 요약하기
            summarizeButton

            // 즐겨찾기 토글 버튼
            favoriteButton
        }
    }

    @ViewBuilder
    private var favoriteButton: some View {
        let isLiked = favoritesFeature.isLiked(feedItem.url.absoluteString)
        Button {
            guard !favoriteLoading else { return }
            Task {
                favoriteLoading = true
                defer { favoriteLoading = false }
                if isLiked {
                    await favoritesFeature.removeFavorite(url: feedItem.url.absoluteString)
                } else {
                    // step-5 K: phase.done에서 summary/insight 꺼내 전달
                    let summary = feature.phase.summaryResult?.summary
                    let insight = feature.phase.summaryResult?.insight
                    await favoritesFeature.addFavorite(
                        feedItem: feedItem,
                        summary: summary,
                        insight: insight
                    )
                }
            }
        } label: {
            HStack {
                Image(systemName: isLiked ? "star.fill" : "star")
                    .foregroundStyle(isLiked ? .yellow : .primary)
                Text(isLiked ? "즐겨찾기 해제" : "즐겨찾기 추가")
            }
            .frame(maxWidth: .infinity)
        }
        .buttonStyle(.bordered)
        .disabled(favoriteLoading)
        .opacity(favoriteLoading ? 0.5 : 1.0)
    }

    @ViewBuilder
    private var summarizeButton: some View {
        switch feature.phase {
        case .idle:
            Button {
                Task { await feature.loadSummary() }
            } label: {
                HStack {
                    Image(systemName: "sparkles")
                    Text("요약하기")
                }
                .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent)

        case .loading:
            HStack {
                ProgressView()
                    .padding(.trailing, 4)
                Text("요약 중...")
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 10)

        case .done:
            Button {
                // 이미 done 상태 — 재요약 불필요
            } label: {
                HStack {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundStyle(.green)
                    Text("요약 완료")
                }
                .frame(maxWidth: .infinity)
            }
            .buttonStyle(.bordered)
            .disabled(true)

        case .failed:
            Button {
                Task { await feature.loadSummary() }
            } label: {
                HStack {
                    Image(systemName: "arrow.clockwise")
                    Text("다시 시도")
                }
                .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent)
            .tint(.orange)
        }
    }
}

// MARK: - Summary Section

extension ArticleDetailView {
    @ViewBuilder
    private var summarySection: some View {
        switch feature.phase {
        case .done(let result):
            VStack(alignment: .leading, spacing: 16) {
                Divider()

                VStack(alignment: .leading, spacing: 8) {
                    Text("요약")
                        .font(.subheadline)
                        .fontWeight(.bold)
                        .foregroundStyle(.secondary)

                    Text(result.summary)
                        .font(.body)
                }

                VStack(alignment: .leading, spacing: 8) {
                    Text("인사이트")
                        .font(.subheadline)
                        .fontWeight(.bold)
                        .foregroundStyle(.secondary)

                    Text(result.insight)
                        .font(.body)
                        .foregroundStyle(.secondary)
                }
            }

        case .failed(let message):
            Text(message)
                .font(.caption)
                .foregroundStyle(.red)
                .padding(.top, 4)

        default:
            EmptyView()
        }
    }
}
