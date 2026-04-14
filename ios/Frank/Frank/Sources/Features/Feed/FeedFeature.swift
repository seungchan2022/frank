import Foundation
import Observation

/// MVP5 M1: FeedFeature — 검색 API 직접 호출 (DB 저장 없음).
/// CollectPort 의존성 제거. 페이지네이션/캐시 제거 (ephemeral 피드).
/// pull-to-refresh = 동일 API 재호출.
///
/// MVP6 M3: 탭별 캐시 (tagCache) + 초기 프리패치.
/// - loadInitial 시 구독 태그 전체를 병렬 프리패치 → 탭 전환 항상 즉시 표시
/// - pull-to-refresh만 현재 탭 캐시 갱신 + 재요청
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

    // MARK: - Cache

    /// 탭별 피드 캐시. 키: tagId?.uuidString ?? "all"
    private var tagCache: [String: [FeedItem]] = [:]

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
            await selectTag(tagId)
        case .refresh:
            await refresh()
        case .reloadAfterTagChange:
            await reloadAfterTagChange()
        }
    }

    // MARK: - Computed (server-filtered — no client-side filtering)

    /// 현재 피드 아이템. 서버에서 이미 태그 필터링된 결과.
    var articles: [FeedItem] { feedItems }

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

    // MARK: - Cache Helpers

    private func cacheKey(for tagId: UUID?) -> String {
        tagId?.uuidString ?? "all"
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

            let items = try await article.fetchFeed(tagId: nil, noCache: false)
            feedItems = items
            tagCache["all"] = items

            // 구독 태그 전체 병렬 프리패치 → 탭 전환 항상 즉시 표시
            await withTaskGroup(of: (String, [FeedItem]?).self) { group in
                for tagId in myTagIds {
                    group.addTask {
                        let key = tagId.uuidString
                        let result = try? await self.article.fetchFeed(tagId: tagId, noCache: false)
                        return (key, result)
                    }
                }
                for await (key, result) in group {
                    if let result {
                        tagCache[key] = result
                    }
                }
            }

            selectedTagId = nil
            phase = .idle
        } catch {
            failLoading("피드를 불러오지 못했습니다. 다시 시도해주세요.")
        }
    }

    private func selectTag(_ tagId: UUID?) async {
        guard phase == .idle else { return }

        let key = cacheKey(for: tagId)

        // 캐시 히트 → 즉시 표시, 재요청 없음
        if let cached = tagCache[key] {
            feedItems = cached
            selectedTagId = tagId
            return
        }

        // 캐시 미스 → 조용히 fetch (로딩 표시 없음, 프리패치 실패 시 fallback)
        do {
            let items = try await article.fetchFeed(tagId: tagId, noCache: false)
            feedItems = items
            tagCache[key] = items
            selectedTagId = tagId
        } catch {
            errorMessage = "태그 피드를 불러오지 못했습니다."
        }
    }

    private func refresh() async {
        guard phase == .idle else { return }
        beginRefresh()

        let key = cacheKey(for: selectedTagId)

        do {
            // MVP10 M3: pull-to-refresh는 서버 TTL 캐시 우회
            let items = try await article.fetchFeed(tagId: selectedTagId, noCache: true)
            feedItems = items
            tagCache[key] = items
            finishRefresh()
        } catch {
            failRefresh("새로고침에 실패했습니다.")
        }
    }

    private func reloadAfterTagChange() async {
        selectedTagId = nil
        tagCache = [:]
        hasLoadedInitially = false
        await loadInitial()
    }
}
