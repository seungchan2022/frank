import SwiftUI

/// MVP5 M1: FeedView — ephemeral 피드 표시.
/// MVP7 M2: LikesFeature 주입 — 카드별 하트 버튼 + 상세 공유.
/// MVP8 M2: RelatedPort 제거, QuizPort 주입 — ArticleDetailView로 퀴즈 포트 전달.
/// MVP14 M3: DEBT-04 — ZStack + onTapGesture로 좋아요/내비게이션 분리.
///           DEBT-05 — DragGesture → TabView(.page) 전환. 각 페이지 독립 캐시 표시.
struct FeedView: View {
    let feature: FeedFeature
    let summarize: any SummarizePort
    let favoritesFeature: FavoritesFeature
    let likesFeature: LikesFeature
    let quiz: any QuizPort
    var onSettingsTapped: (() -> Void)?

    @State private var navigationPath = NavigationPath()
    @State private var pageIndex: Int = 0
    @State private var scrollPageID: Int? = 0

    private var allTagIds: [UUID?] {
        [nil] + feature.tags.map { Optional($0.id) }
    }

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

                // 초기 전체 로딩 중에는 ShimmerList 단독 표시 (ScrollView 겹침 방지)
                if feature.phase == .initialLoading {
                    List { ShimmerListView() }.listStyle(.plain)
                } else {
                    ScrollView(.horizontal, showsIndicators: false) {
                        LazyHStack(spacing: 0) {
                            ForEach(0..<allTagIds.count, id: \.self) { index in
                                pageContent(index: index)
                                    .containerRelativeFrame(.horizontal)
                                    .id(index)
                            }
                        }
                        .scrollTargetLayout()
                    }
                    .scrollTargetBehavior(.viewAligned(limitBehavior: .always))
                    .scrollPosition(id: $scrollPageID)
                    .onChange(of: scrollPageID) { _, newID in
                        guard let newIndex = newID, newIndex < allTagIds.count else { return }
                        if pageIndex != newIndex { pageIndex = newIndex }
                        Task { await feature.send(.selectTag(allTagIds[newIndex])) }
                    }
                    .onChange(of: feature.selectedTagId) { _, newTagId in
                        let newIndex = allTagIds.firstIndex(where: { $0 == newTagId }) ?? 0
                        scrollPageID = newIndex
                        pageIndex = newIndex
                    }
                }
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

    // MARK: - Per-Page Content

    @ViewBuilder
    private func pageContent(index: Int) -> some View {
        let tagId = allTagIds[index]
        let isCurrent = index == pageIndex

        if feature.isLoadingTag(tagId) {
            List { ShimmerListView() }.listStyle(.plain)
        } else {
            let items = feature.items(for: tagId)
            if items.isEmpty {
                EmptyStateView()
            } else {
                articleList(items: items, isCurrent: isCurrent)
            }
        }
    }

    // MARK: - Article List

    private func articleList(items: [FeedItem], isCurrent: Bool) -> some View {
        List {
            ForEach(items) { item in
                let isLiked = likesFeature.isLiked(item.url.absoluteString)
                ZStack(alignment: .bottomTrailing) {
                    ArticleCardView(article: item)

                    Button {
                        Task { await likesFeature.like(feedItem: item) }
                    } label: {
                        Image(systemName: isLiked ? "heart.fill" : "heart")
                            .foregroundStyle(isLiked ? .red : .gray)
                            .font(.system(size: 20))
                            .frame(width: 44, height: 44)
                    }
                    .buttonStyle(.plain)
                    .accessibilityLabel(isLiked ? "추천 완료" : "추천에 반영")
                    .accessibilityIdentifier("feed_like_button")
                }
                .contentShape(Rectangle())
                .onTapGesture {
                    navigationPath.append(item.id)
                }
                .listRowInsets(EdgeInsets())
            }

            // ST5: 무한 스크롤 sentinel — 현재 페이지에만 표시
            if isCurrent {
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
        }
        .listStyle(.plain)
        .refreshable {
            await feature.send(.refresh)
        }
    }
}
