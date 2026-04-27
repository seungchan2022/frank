import SwiftUI

/// MVP5 M1: FeedView — ephemeral 피드 표시.
/// collectAndRefresh 버튼 제거. pull-to-refresh = API 재호출.
/// NavigationLink value: String (FeedItem.id = url absoluteString 기반).
/// MVP7 M2: LikesFeature 주입 — 카드별 하트 버튼 + 상세 공유.
/// MVP8 M2: RelatedPort 제거, QuizPort 주입 — ArticleDetailView로 퀴즈 포트 전달.
struct FeedView: View {
    let feature: FeedFeature
    let summarize: any SummarizePort
    let favoritesFeature: FavoritesFeature
    let likesFeature: LikesFeature
    let quiz: any QuizPort
    var onSettingsTapped: (() -> Void)?

    @State private var navigationPath = NavigationPath()

    var body: some View {
        NavigationStack(path: $navigationPath) {
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
                    ArticleDetailView(
                        feedItem: item,
                        summarize: summarize,
                        favoritesFeature: favoritesFeature,
                        likesFeature: likesFeature,
                        quiz: quiz
                    )
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
                // NavigationLink 대신 onTapGesture + overlay 분리:
                // List 내 NavigationLink는 전체 row 터치를 가져가
                // 오버레이 Button 탭도 동시에 발동하는 버그가 있음.
                ArticleCardView(article: item)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        navigationPath.append(item.id)
                    }
                    .overlay(alignment: .bottomTrailing) {
                        Button {
                            Task { await likesFeature.like(feedItem: item) }
                        } label: {
                            Image(systemName: likesFeature.isLiked(item.url.absoluteString) ? "heart.fill" : "heart")
                                .foregroundStyle(likesFeature.isLiked(item.url.absoluteString) ? .red : .gray)
                                .font(.system(size: 18))
                                .padding(8)
                        }
                        .buttonStyle(.plain)
                        .accessibilityLabel(likesFeature.isLiked(item.url.absoluteString) ? "추천 완료" : "추천에 반영")
                    }
                    .listRowInsets(EdgeInsets())
            }

            // ST5: 무한 스크롤 sentinel
            if feature.isLoadingMore {
                HStack {
                    Spacer()
                    ProgressView()
                    Spacer()
                }
                .listRowSeparator(.hidden)
                .listRowInsets(EdgeInsets(top: 8, leading: 0, bottom: 8, trailing: 0))
            } else if !feature.hasMore {
                Text("모든 기사를 읽었습니다")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .listRowSeparator(.hidden)
                    .listRowInsets(EdgeInsets(top: 8, leading: 0, bottom: 8, trailing: 0))
            } else {
                Color.clear
                    .frame(height: 1)
                    .listRowSeparator(.hidden)
                    .listRowInsets(EdgeInsets())
                    .onAppear {
                        Task { await feature.send(.loadMore) }
                    }
            }
        }
        .listStyle(.plain)
        .refreshable {
            await feature.send(.refresh)
        }
    }
}
