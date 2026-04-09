import SwiftUI

/// MVP5 M2: ArticleDetailView — 온디맨드 요약 UI.
/// - 요약하기 버튼: idle/loading/done/failed 상태에 따라 UI 전환
/// - 즐겨찾기 버튼: M3에서 구현 (레이아웃 예약만)
struct ArticleDetailView: View {
    let feedItem: FeedItem
    private let summarizePort: any SummarizePort
    @State private var feature: ArticleDetailFeature

    @Environment(\.openURL) private var openURL

    init(feedItem: FeedItem, summarize: any SummarizePort) {
        self.feedItem = feedItem
        self.summarizePort = summarize
        self._feature = State(initialValue: ArticleDetailFeature(feedItem: feedItem, summarize: summarize))
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                headerSection
                Divider()
                snippetSection
                actionButtons
                summarySection
            }
            .padding(.horizontal, 20)
            .padding(.vertical, 16)
        }
        .navigationBarTitleDisplayMode(.inline)
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
                openURL(feedItem.url)
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

            // 즐겨찾기 (M3 예약 — 비활성)
            Button {
                // M3에서 구현
            } label: {
                HStack {
                    Image(systemName: "star")
                    Text("즐겨찾기")
                }
                .frame(maxWidth: .infinity)
            }
            .buttonStyle(.bordered)
            .disabled(true)
            .opacity(0.4)
        }
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
