import SwiftUI

/// MVP5 M3: FavoritesView — 스크랩 목록 탭.
/// step-5 L 반영: FavoriteItem.summary/insight → SummarySessionCache에 주입
/// MVP8 M2: RelatedPort 제거, QuizPort 주입 — ArticleDetailView로 퀴즈 포트 전달.
/// MVP8 M3: WrongAnswerPort + FavoritesPort 주입 — QuizFeature에 전달.
///           세그먼트 탭 ("기사" / "오답 노트") + quizCompleted 배지 + WrongAnswersFeature 통합.
/// MVP11 M4: TagChipBarView 통합 — 기사·오답 탭 공유 selectedTagId 필터.
struct FavoritesView: View {
    let feature: FavoritesFeature
    let summarize: any SummarizePort
    let likesFeature: LikesFeature
    let quiz: any QuizPort
    let wrongAnswer: any WrongAnswerPort
    let favorites: any FavoritesPort

    @State private var selectedTab: FavoritesTab = .articles
    @State private var wrongAnswersFeature: WrongAnswersFeature

    /// MVP11 M4: 기사·오답 탭 공유 selectedTagId. 탭 전환 시 nil 초기화.
    /// MVP11 M4: 기사·오답 탭 공유 선택 태그. 탭 전환 시 nil 초기화.
    @State private var selectedTagId: UUID?

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

    // MARK: - MVP11 M4: 오답 탭 필터 computed

    /// url → tagId 매핑. tagId가 없는 즐겨찾기는 포함하지 않음.
    var wrongAnswerTagMap: [String: UUID] {
        WrongAnswerTagFilter.buildTagMap(from: feature.items)
    }

    /// 오답 탭에 표시할 태그 칩 소스.
    /// wrongAnswersFeature.items의 articleUrl → tagMap → tagId 조인으로 실제 오답 기사에 존재하는 태그만 표시.
    /// BUG-F 수정: feature.tags(즐겨찾기 전체 태그)가 아닌, 오답 기사 URL에 해당하는 태그만 표시.
    var wrongAnswerTags: [Tag] {
        let map = wrongAnswerTagMap
        let usedTagIds = Set(wrongAnswersFeature.items.compactMap { map[$0.articleUrl] })
        return feature.tags.filter { usedTagIds.contains($0.id) }
    }

    /// 오답 탭 필터 결과.
    /// - selectedTagId == nil: 전체 반환
    /// - selectedTagId != nil: 해당 태그 url Set 교집합 (favorites 미등록 오답은 제외)
    var filteredWrongAnswers: [WrongAnswer] {
        WrongAnswerTagFilter.apply(
            items: wrongAnswersFeature.items,
            tagMap: wrongAnswerTagMap,
            selectedTagId: selectedTagId
        )
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
            .task { await feature.loadFavorites() }
            .task(id: selectedTab) {
                selectedTagId = nil
                if selectedTab == .wrongAnswers {
                    await wrongAnswersFeature.load()
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
                    tagChipBar(tags: feature.tags) { selectedTagId = $0 }
                    itemList
                }
            }
        }
    }

    private var itemList: some View {
        List(feature.filteredItems(selectedTagId: selectedTagId)) { item in
            NavigationLink(value: item) {
                FavoriteRowView(item: item)
            }
            .swipeActions(edge: .trailing) {
                Button(role: .destructive) {
                    Task {
                        await feature.removeFavorite(url: item.url)
                        // ST2 BUG-E: 삭제 후 남은 아이템에 현재 태그가 없으면 selectedTagId 초기화
                        if FavoritesFeature.shouldResetTagId(remaining: feature.items, current: selectedTagId) {
                            selectedTagId = nil
                        }
                    }
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
                    tagChipBar(tags: wrongAnswerTags, onSelect: { selectedTagId = $0 }) { newTags in
                        // 현재 선택된 태그가 오답 탭에서 유효하지 않으면 nil로 초기화
                        if let current = selectedTagId,
                           !newTags.contains(where: { $0.id == current }) {
                            selectedTagId = nil
                        }
                    }
                    wrongAnswersList
                }
            }
        }
    }

    private var wrongAnswersList: some View {
        List {
            ForEach(filteredWrongAnswers) { item in
                WrongAnswerRow(item: item)
                    .swipeActions(edge: .trailing) {
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

    // MARK: - Tag Chip Bars

    /// TagChipBarView + Divider 블록 공용 헬퍼.
    /// - Parameters:
    ///   - tags: 표시할 태그 목록
    ///   - onSelect: 태그 선택 콜백
    ///   - onTagsChanged: tags 변경 시 현재 선택 태그 유효성 검사가 필요한 경우 주입.
    ///                    nil이면 onChange 없이 렌더링 (기사 탭).
    @ViewBuilder
    private func tagChipBar(
        tags: [Tag],
        onSelect: @escaping (UUID?) -> Void,
        onTagsChanged: (([Tag]) -> Void)? = nil
    ) -> some View {
        if !tags.isEmpty {
            if let onTagsChanged {
                TagChipBarView(tags: tags, selectedTagId: selectedTagId, onSelect: onSelect)
                    .padding(.vertical, 8)
                    .onChange(of: tags) { _, newTags in onTagsChanged(newTags) }
            } else {
                TagChipBarView(tags: tags, selectedTagId: selectedTagId, onSelect: onSelect)
                    .padding(.vertical, 8)
            }
            Divider()
        }
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
