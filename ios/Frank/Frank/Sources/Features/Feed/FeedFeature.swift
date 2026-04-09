import Foundation
import Observation

enum FeedAction {
    case loadInitial
    case selectTag(UUID?)
    case loadMore
    case refresh
    case collectAndRefresh
    case reloadAfterTagChange
}

/// Feed의 주(主) 로딩 phase. 동시에 한 가지 phase만 가질 수 있다.
///
/// `isLoadingMore`는 phase와 직교(병행 가능)하므로 별도 변수로 둔다.
enum LoadingPhase: Equatable, Sendable {
    case idle
    case initialLoading
    case collecting
    case refreshing
}

@Observable
@MainActor
final class FeedFeature {

    // MARK: - Data

    private(set) var tags: [Tag] = []
    private(set) var selectedTagId: UUID?
    private(set) var articles: [Article] = []

    // MARK: - Loading State

    /// 주(主) phase — `idle / initialLoading / collecting / refreshing` 중 하나
    private(set) var phase: LoadingPhase = .idle
    /// 페이지네이션 추가 로딩 — phase와 직교(병행 가능)
    private(set) var isLoadingMore = false
    private(set) var hasMore = true

    // MARK: - Derived (편의 — 뷰/테스트에서 phase 비교 단축)

    var isLoading: Bool { phase == .initialLoading }
    var isCollecting: Bool { phase == .collecting }
    var isRefreshing: Bool { phase == .refreshing }

    // MARK: - Error

    private(set) var errorMessage: String?

    // MARK: - Cache

    private var cache: [UUID?: [Article]] = [:]
    private var offsetCache: [UUID?: Int] = [:]
    private var hasMoreCache: [UUID?: Bool] = [:]

    // MARK: - Constants

    private let pageSize = 20

    // MARK: - Dependencies

    private let article: any ArticlePort
    private let collect: any CollectPort
    private let tag: any TagPort

    // MARK: - Init

    init(article: any ArticlePort, collect: any CollectPort, tag: any TagPort) {
        self.article = article
        self.collect = collect
        self.tag = tag
    }

    // MARK: - Send

    func send(_ action: FeedAction) async {
        switch action {
        case .loadInitial:
            await loadInitial()
        case .selectTag(let tagId):
            await selectTag(tagId)
        case .loadMore:
            await loadMore()
        case .refresh:
            await refresh()
        case .collectAndRefresh:
            await collectAndRefresh()
        case .reloadAfterTagChange:
            await reloadAfterTagChange()
        }
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

    private func beginCollect() {
        phase = .collecting
        errorMessage = nil
    }

    private func finishCollect() {
        phase = .idle
    }

    private func failCollect(_ message: String) {
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

    // MARK: - Fetch + Cache Helper

    /// 지정된 tagId로 첫 페이지를 fetch하고, articles를 교체, 캐시를 갱신한다.
    @discardableResult
    private func fetchAndCache(tagId: UUID?) async throws -> [Article] {
        let fetched = try await article.fetchArticles(
            filter: ArticleFilter(tagId: tagId, limit: pageSize, offset: 0)
        )
        // 전체 탭(tagId=nil)일 때 현재 구독 태그로 필터링
        let myTagIds = Set(tags.map(\.id))
        let filtered = tagId == nil && !myTagIds.isEmpty
            ? fetched.filter { item in
                guard let itemTagId = item.tagId else { return true }
                return myTagIds.contains(itemTagId)
            }
            : fetched
        articles = filtered
        cache[tagId] = filtered
        offsetCache[tagId] = fetched.count
        hasMore = fetched.count >= pageSize
        hasMoreCache[tagId] = hasMore
        return filtered
    }

    /// 캐시 전체 무효화
    private func invalidateCache() {
        cache.removeAll()
        offsetCache.removeAll()
        hasMoreCache.removeAll()
    }

    // MARK: - Core Logic

    private func loadInitial() async {
        beginLoading()
        do {
            let allTags = try await tag.fetchAllTags()
            let myTagIds = try await tag.fetchMyTagIds()
            tags = allTags.filter { myTagIds.contains($0.id) }

            let fetched = try await fetchAndCache(tagId: selectedTagId)
            phase = .idle

            // 기사 0건이면 자동 수집
            if fetched.isEmpty {
                await collectAndRefresh()
            }
        } catch {
            failLoading("피드를 불러오지 못했습니다. 다시 시도해주세요.")
        }
    }

    private func selectTag(_ tagId: UUID?) async {
        selectedTagId = tagId

        // 캐시 히트
        if let cached = cache[tagId] {
            articles = cached
            hasMore = hasMoreCache[tagId] ?? true
            return
        }

        // 캐시 미스 → fetch
        do {
            try await fetchAndCache(tagId: tagId)
        } catch {
            errorMessage = "기사를 불러오지 못했습니다."
        }
    }

    private func loadMore() async {
        // 다른 phase 진행 중이면 무시 — race 방지 (캐시/articles 덮어쓰기 차단)
        guard !isLoadingMore, hasMore, phase == .idle else { return }

        isLoadingMore = true
        let offset = offsetCache[selectedTagId] ?? 0

        do {
            let fetched = try await article.fetchArticles(
                filter: ArticleFilter(tagId: selectedTagId, limit: pageSize, offset: offset)
            )
            // 전체 탭(tagId=nil)일 때 현재 구독 태그로 필터링
            let myTagIds = Set(tags.map(\.id))
            let filtered = selectedTagId == nil && !myTagIds.isEmpty
                ? fetched.filter { item in
                    guard let itemTagId = item.tagId else { return true }
                    return myTagIds.contains(itemTagId)
                }
                : fetched
            articles += filtered
            cache[selectedTagId] = articles
            offsetCache[selectedTagId] = offset + fetched.count
            hasMore = fetched.count >= pageSize
            hasMoreCache[selectedTagId] = hasMore
            isLoadingMore = false
        } catch {
            isLoadingMore = false
            errorMessage = "추가 기사를 불러오지 못했습니다."
        }
    }

    private func collectAndRefresh() async {
        guard phase == .idle else { return }

        beginCollect()
        do {
            // Step 1: 수집
            _ = try await collect.triggerCollect()

            // Step 2: 수집 후 기사 fetch (메타데이터만)
            invalidateCache()
            try await fetchAndCache(tagId: selectedTagId)

            finishCollect()
        } catch {
            failCollect("수집에 실패했습니다. 다시 시도해주세요.")
        }
    }

    private func refresh() async {
        // 다른 phase 진행 중이면 무시 — refresh가 collect 진행을 덮어쓰는 것 차단
        guard phase == .idle else { return }
        beginRefresh()
        invalidateCache()
        do {
            try await fetchAndCache(tagId: selectedTagId)
            finishRefresh()
        } catch {
            failRefresh("새로고침에 실패했습니다.")
        }
    }

    private func reloadAfterTagChange() async {
        invalidateCache()
        selectedTagId = nil
        await loadInitial()
    }
}
