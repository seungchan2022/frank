import SwiftUI

/// MVP5 M3: FavoritesView — 스크랩 목록 탭.
/// step-5 L 반영: FavoriteItem.summary/insight → SummarySessionCache에 주입
/// MVP8 M2: RelatedPort 제거, QuizPort 주입 — ArticleDetailView로 퀴즈 포트 전달.
/// MVP8 M3: WrongAnswerPort + FavoritesPort 주입 — QuizFeature에 전달.
///           세그먼트 탭 ("기사" / "오답 노트") + quizCompleted 배지 + WrongAnswersFeature 통합.
/// MVP11 M4: TagChipBarView 통합 — 기사·오답 탭 공유 selectedTagId 필터.
/// MVP14 M3: DEBT-05 — swipeActions → contextMenu 전환, DragGesture → TabView(.page) 전환.
struct FavoritesView: View {
    let feature: FavoritesFeature
    let summarize: any SummarizePort
    let likesFeature: LikesFeature
    let quiz: any QuizPort
    let wrongAnswer: any WrongAnswerPort
    let favorites: any FavoritesPort

    @State private var selectedTab: FavoritesTab = .articles
    @State private var wrongAnswersFeature: WrongAnswersFeature

    @State private var articlesPageIndex: Int = 0
    @State private var articlesScrollID: Int? = 0
    @State private var wrongAnswersPageIndex: Int = 0
    @State private var wrongAnswersScrollID: Int? = 0

    init(
        feature: FavoritesFeature,
        summarize: any SummarizePort,
        likesFeature: LikesFeature,
        quiz: any QuizPort,
        wrongAnswer: any WrongAnswerPort,
        favorites: any FavoritesPort
    ) {
        self.feature = feature
        self.summarize = summarize
        self.likesFeature = likesFeature
        self.quiz = quiz
        self.wrongAnswer = wrongAnswer
        self.favorites = favorites
        self._wrongAnswersFeature = State(initialValue: WrongAnswersFeature(wrongAnswer: wrongAnswer))
    }

    // MARK: - Tag ID Arrays

    var articlesTagIds: [UUID?] {
        [nil] + feature.tags.map { Optional($0.id) }
    }

    var wrongAnswerTagIds: [UUID?] {
        [nil] + wrongAnswerTags.map { Optional($0.id) }
    }

    var articlesSelectedTagId: UUID? {
        articlesPageIndex < articlesTagIds.count ? articlesTagIds[articlesPageIndex] : nil
    }

    var wrongAnswersSelectedTagId: UUID? {
        wrongAnswersPageIndex < wrongAnswerTagIds.count ? wrongAnswerTagIds[wrongAnswersPageIndex] : nil
    }

    // MARK: - MVP13 M2: 오답 탭 필터 computed (favorites 브릿지 제거)

    var wrongAnswerTags: [Tag] {
        let usedTagIds = Set(wrongAnswersFeature.items.compactMap { $0.tagId })
        return feature.tags.filter { usedTagIds.contains($0.id) }
    }

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // 세그먼트 컨트롤
                Picker("탭", selection: $selectedTab) {
                    ForEach(FavoritesTab.allCases) { tab in
                        Text(tab.title).tag(tab)
                    }
                }
                .pickerStyle(.segmented)
                .padding(.horizontal, 16)
                .padding(.vertical, 10)

                // 탭별 콘텐츠
                switch selectedTab {
                case .articles:
                    articlesContent
                case .wrongAnswers:
                    wrongAnswersContent
                }
            }
            .navigationTitle("스크랩")
            .navigationDestination(for: FavoriteItem.self) { item in
                favoriteDetail(item: item)
            }
            .task { await feature.loadFavorites() }
            .task(id: selectedTab) {
                articlesPageIndex = 0
                articlesScrollID = 0
                wrongAnswersPageIndex = 0
                wrongAnswersScrollID = 0
                if selectedTab == .wrongAnswers {
                    await wrongAnswersFeature.load()
                }
            }
            .onReceive(NotificationCenter.default.publisher(for: .wrongAnswerSaved)) { _ in
                if selectedTab == .wrongAnswers {
                    Task { await wrongAnswersFeature.load() }
                }
            }
            .overlay(alignment: .bottom) {
                if let errorMsg = feature.operationError {
                    operationErrorBanner(message: errorMsg)
                }
            }
            .overlay(alignment: .bottom) {
                if let errorMsg = wrongAnswersFeature.deleteError {
                    operationErrorBanner(message: errorMsg)
                        .onTapGesture { wrongAnswersFeature.clearDeleteError() }
                }
            }
        }
    }

    // MARK: - 기사 탭

    @ViewBuilder
    private var articlesContent: some View {
        switch feature.phase {
        case .loading:
            loadingView

        case .failed(let message):
            errorView(message: message) {
                Task { await feature.loadFavorites() }
            }

        case .idle, .done:
            if feature.items.isEmpty && feature.hasLoaded {
                articlesEmptyView
            } else {
                VStack(spacing: 0) {
                    if !feature.tags.isEmpty {
                        TagChipBarView(
                            tags: feature.tags,
                            selectedTagId: articlesSelectedTagId,
                            onSelect: { newTagId in
                                let idx = articlesTagIds.firstIndex(where: { $0 == newTagId }) ?? 0
                                articlesPageIndex = idx
                                articlesScrollID = idx
                            }
                        )
                        .padding(.vertical, 8)
                        Divider()
                    }
                    ScrollView(.horizontal, showsIndicators: false) {
                        LazyHStack(spacing: 0) {
                            ForEach(0..<articlesTagIds.count, id: \.self) { index in
                                itemListForTag(articlesTagIds[index])
                                    .containerRelativeFrame(.horizontal)
                                    .id(index)
                            }
                        }
                        .scrollTargetLayout()
                    }
                    .scrollTargetBehavior(.viewAligned(limitBehavior: .always))
                    .scrollPosition(id: $articlesScrollID)
                    .onChange(of: articlesScrollID) { _, newID in
                        if let idx = newID { articlesPageIndex = idx }
                    }
                }
            }
        }
    }

    private func itemListForTag(_ tagId: UUID?) -> some View {
        List(feature.filteredItems(selectedTagId: tagId)) { item in
            NavigationLink(value: item) {
                FavoriteRowView(item: item)
            }
            .contextMenu {
                Button(role: .destructive) {
                    Task {
                        await feature.removeFavorite(url: item.url)
                        if FavoritesFeature.shouldResetTagId(remaining: feature.items, current: tagId) {
                            articlesPageIndex = 0
                        }
                    }
                } label: {
                    Label("삭제", systemImage: "trash")
                }
            }
        }
        .listStyle(.plain)
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
                snippet: item.snippet,
                imageUrl: item.imageUrl.flatMap { URL(string: $0) }
            )
            let _ = injectSummaryCache(item: item, url: url.absoluteString)
            ArticleDetailView(
                feedItem: feedItem,
                summarize: summarize,
                favoritesFeature: feature,
                likesFeature: likesFeature,
                quiz: quiz,
                wrongAnswer: wrongAnswer,
                favorites: favorites
            )
        }
    }

    // MARK: - 오답 노트 탭

    @ViewBuilder
    private var wrongAnswersContent: some View {
        switch wrongAnswersFeature.phase {
        case .idle, .loading:
            loadingView

        case .failed(let message):
            errorView(message: message) {
                Task { await wrongAnswersFeature.load() }
            }

        case .done:
            if wrongAnswersFeature.items.isEmpty {
                wrongAnswersEmptyView
            } else {
                VStack(spacing: 0) {
                    if !wrongAnswerTags.isEmpty {
                        TagChipBarView(
                            tags: wrongAnswerTags,
                            selectedTagId: wrongAnswersSelectedTagId,
                            onSelect: { newTagId in
                                let idx = wrongAnswerTagIds.firstIndex(where: { $0 == newTagId }) ?? 0
                                wrongAnswersPageIndex = idx
                            }
                        )
                        .padding(.vertical, 8)
                        Divider()
                    }
                    ScrollView(.horizontal, showsIndicators: false) {
                        LazyHStack(spacing: 0) {
                            ForEach(0..<wrongAnswerTagIds.count, id: \.self) { index in
                                wrongAnswersListForTag(wrongAnswerTagIds[index])
                                    .containerRelativeFrame(.horizontal)
                                    .id(index)
                            }
                        }
                        .scrollTargetLayout()
                    }
                    .scrollTargetBehavior(.viewAligned(limitBehavior: .always))
                    .scrollPosition(id: $wrongAnswersScrollID)
                    .onChange(of: wrongAnswersScrollID) { _, newID in
                        if let idx = newID { wrongAnswersPageIndex = idx }
                    }
                    .onChange(of: wrongAnswerTags) { _, newTags in
                        if wrongAnswersPageIndex > newTags.count {
                            wrongAnswersPageIndex = 0
                            wrongAnswersScrollID = 0
                        }
                    }
                }
            }
        }
    }

    private func wrongAnswersListForTag(_ tagId: UUID?) -> some View {
        let items = WrongAnswerTagFilter.filter(
            items: wrongAnswersFeature.items,
            selectedTagId: tagId
        )
        return List {
            ForEach(items) { item in
                WrongAnswerRow(item: item)
                    .contextMenu {
                        Button(role: .destructive) {
                            Task { await wrongAnswersFeature.delete(id: item.id) }
                        } label: {
                            Label("삭제", systemImage: "trash")
                        }
                    }
            }
        }
        .listStyle(.plain)
    }

    // MARK: - Empty States

    private var articlesEmptyView: some View {
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

    private var wrongAnswersEmptyView: some View {
        VStack(spacing: 12) {
            Image(systemName: "checkmark.seal")
                .font(.system(size: 48))
                .foregroundStyle(.indigo)
            Text("오답이 없습니다")
                .font(.headline)
            Text("퀴즈를 풀고 틀린 문제를 여기서 복습해보세요.")
                .font(.caption)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding()
    }

    // MARK: - Shared Views

    private var loadingView: some View {
        VStack(spacing: 12) {
            ProgressView()
            Text("불러오는 중...")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func errorView(message: String, retry: @escaping () -> Void) -> some View {
        VStack(spacing: 12) {
            Image(systemName: "exclamationmark.triangle")
                .font(.largeTitle)
                .foregroundStyle(.orange)
            Text(message)
                .font(.body)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
            Button("다시 시도", action: retry)
                .buttonStyle(.bordered)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding()
    }

    // MARK: - Operation Error Banner

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

    @discardableResult
    private func injectSummaryCache(item: FavoriteItem, url: String) -> Bool {
        if let summary = item.summary, let insight = item.insight {
            SummarySessionCache.shared.set(url, SummaryResult(summary: summary, insight: insight))
            return true
        }
        return false
    }
}

// MARK: - Tab Model

enum FavoritesTab: String, CaseIterable, Identifiable {
    case articles
    case wrongAnswers

    var id: String { rawValue }

    var title: String {
        switch self {
        case .articles: return "기사"
        case .wrongAnswers: return "오답 노트"
        }
    }
}

/// 즐겨찾기 목록 행 뷰 — MVP6 M1: ArticleCardView와 동일한 썸네일 레이아웃.
struct FavoriteRowView: View {
    let item: FavoriteItem

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            // 썸네일 영역 (72×72)
            thumbnailView

            // 텍스트 영역
            VStack(alignment: .leading, spacing: 6) {
                Text(item.title)
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .foregroundStyle(.primary)
                    .lineLimit(2)

                HStack(spacing: 4) {
                    Text(item.source)
                    if let createdAt = item.createdAt {
                        Text("·")
                        Text(ArticleCardView.relativeTimeText(createdAt))
                    }
                    Spacer()
                    if item.summary != nil {
                        Image(systemName: "text.quote")
                            .foregroundStyle(.indigo)
                    }
                    if item.quizCompleted {
                        Text("퀴즈 ✓")
                            .font(.caption2)
                            .fontWeight(.semibold)
                            .foregroundStyle(.white)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(Color.indigo)
                            .clipShape(Capsule())
                    }
                }
                .font(.caption)
                .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(.vertical, 8)
    }

    @ViewBuilder
    private var thumbnailView: some View {
        if let imageUrl = item.imageUrl.flatMap({ URL(string: $0) }) {
            AsyncImage(url: imageUrl) { phase in
                switch phase {
                case .success(let image):
                    image
                        .resizable()
                        .scaledToFill()
                        .frame(width: 72, height: 72)
                        .clipShape(RoundedRectangle(cornerRadius: 8))
                default:
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
