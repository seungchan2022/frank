import Foundation
import Observation

/// MVP5 M1: FeedFeature — 검색 API 직접 호출 (DB 저장 없음).
/// CollectPort 의존성 제거. 페이지네이션/캐시 제거 (ephemeral 피드).
/// pull-to-refresh = 동일 API 재호출.
enum FeedAction {
    case loadInitial
    case selectTag(UUID?)
    case refresh
    case reloadAfterTagChange
}

/// Feed의 주(主) 로딩 phase. 동시에 한 가지 phase만 가질 수 있다.
enum LoadingPhase: Equatable, Sendable {
    case idle
    case initialLoading
    case refreshing
}

@Observable
@MainActor
final class FeedFeature {

    // MARK: - Data

    private(set) var tags: [Tag] = []
    private(set) var selectedTagId: UUID?
    private(set) var feedItems: [FeedItem] = []

    // MARK: - Loading State

    /// 주(主) phase — `idle / initialLoading / refreshing` 중 하나
    private(set) var phase: LoadingPhase = .idle

    // MARK: - Derived

    var isLoading: Bool { phase == .initialLoading }
    var isRefreshing: Bool { phase == .refreshing }

    // MARK: - Error

    private(set) var errorMessage: String?

    // MARK: - Dependencies

    private let article: any ArticlePort
    private let tag: any TagPort

    /// loadInitial이 이미 한 번 이상 완료됐는지 추적.
    /// .task 재실행(뒤로가기 복귀 등)에 의한 중복 API 호출 방지.
    private var hasLoadedInitially = false

    // MARK: - Init

    init(article: any ArticlePort, tag: any TagPort) {
        self.article = article
        self.tag = tag
    }

    // MARK: - Send

    func send(_ action: FeedAction) async {
        switch action {
        case .loadInitial:
            await loadInitial()
        case .selectTag(let tagId):
            selectTag(tagId)
        case .refresh:
            await refresh()
        case .reloadAfterTagChange:
            await reloadAfterTagChange()
        }
    }

    // MARK: - Computed (derived from feedItems + selectedTagId)

    /// 현재 선택된 태그 필터에 맞는 피드 아이템 목록
    var articles: [FeedItem] {
        guard let tagId = selectedTagId else { return feedItems }
        return feedItems.filter { $0.tagId == tagId }
    }

    // MARK: - State Transition Helpers

    private func beginLoading() {
        phase = .initialLoading
        errorMessage = nil
    }

    private func failLoading(_ message: String) {
        phase = .idle
        errorMessage = message
    }

    private func beginRefresh() {
        phase = .refreshing
        errorMessage = nil
    }

    private func finishRefresh() {
        phase = .idle
    }

    private func failRefresh(_ message: String) {
        phase = .idle
        errorMessage = message
    }

    // MARK: - Core Logic

    private func loadInitial() async {
        guard !hasLoadedInitially else { return }
        hasLoadedInitially = true
        beginLoading()
        do {
            let allTags = try await tag.fetchAllTags()
            let myTagIds = try await tag.fetchMyTagIds()
            tags = allTags.filter { myTagIds.contains($0.id) }

            feedItems = try await article.fetchFeed()
            phase = .idle
        } catch {
            failLoading("피드를 불러오지 못했습니다. 다시 시도해주세요.")
        }
    }

    private func selectTag(_ tagId: UUID?) {
        selectedTagId = tagId
    }

    private func refresh() async {
        guard phase == .idle else { return }
        beginRefresh()
        do {
            feedItems = try await article.fetchFeed()
            finishRefresh()
        } catch {
            failRefresh("새로고침에 실패했습니다.")
        }
    }

    private func reloadAfterTagChange() async {
        selectedTagId = nil
        hasLoadedInitially = false
        await loadInitial()
    }
}
