import Foundation
import Observation

/// MVP5 M1: FeedFeature — 검색 API 직접 호출 (DB 저장 없음).
/// CollectPort 의존성 제거. 페이지네이션/캐시 제거 (ephemeral 피드).
/// pull-to-refresh = 동일 API 재호출.
///
/// MVP6 M3: 탭별 캐시 (tagCache) + 초기 프리패치.
/// - loadInitial 시 구독 태그 전체를 병렬 프리패치 → 탭 전환 항상 즉시 표시
/// - pull-to-refresh만 현재 탭 캐시 갱신 + 재요청
///
/// MVP12 M3 ST5: 무한 스크롤 (TagState 기반 페이지네이션).
/// - tagCache → tagStates: [String: TagState] 교체
/// - loadMore() → 현재 탭 다음 페이지 로드
/// - 프리패치 제거 (C안): loadInitial은 전체 탭 첫 페이지만 로드
///
/// MVP13 M2: 초기 로드 시 tagId 기준 완전 재분리 저장 (rebuildTagStates) — 탭 전환 즉시 표시.
/// - refresh도 rebuildTagStates로 완전 재구성 — stale 태그 캐시 자동 제거
enum FeedAction {
    case loadInitial
    case selectTag(UUID?)
    case refresh
    case reloadAfterTagChange
    case loadMore
}

/// Feed의 주(主) 로딩 phase. 동시에 한 가지 phase만 가질 수 있다.
enum LoadingPhase: Equatable, Sendable {
    case idle
    case initialLoading
    case refreshing
}

/// 탭별 무한 스크롤 상태.
enum TagStatus: Equatable, Sendable {
    case idle
    case loading      // 첫 페이지 또는 탭 전환 로딩
    case loadingMore  // 추가 페이지 로딩 중
}

/// MVP12 M3 ST5: 탭별 페이지네이션 상태.
struct TagState: Sendable {
    var items: [FeedItem] = []
    var nextOffset: Int = 0
    var hasMore: Bool = true
    var status: TagStatus = .idle

    /// 첫 페이지 로드 결과로 TagState를 생성하는 팩토리.
    /// `loadInitial`, `selectTag`, `refresh` 에서 반복되는 초기화 패턴을 통합.
    static func firstPage(items: [FeedItem], pageSize: Int) -> TagState {
        TagState(
            items: items,
            nextOffset: items.count,
            hasMore: items.count == pageSize,
            status: .idle
        )
    }
}

/// 한 번에 가져올 기사 수. 서버 MAX_FEED_LIMIT(50) 이하.
private let PAGE_SIZE = 20

@Observable
@MainActor
final class FeedFeature {

    // MARK: - Data

    private(set) var tags: [Tag] = []
    private(set) var selectedTagId: UUID?

    // MARK: - Loading State

    /// 주(主) phase — `idle / initialLoading / refreshing` 중 하나
    private(set) var phase: LoadingPhase = .idle

    // MARK: - Derived

    /// 초기 로딩 중이거나 탭 전환 캐시 미스로 인한 로딩 중이면 true.
    /// FeedView는 isLoading = true 시 ShimmerListView를 표시한다.
    /// BUG-008: 탭 전환 캐시 미스 시 빈 TagState(items=[])가 먼저 세팅되면
    /// FeedView가 EmptyStateView를 보여주는 깜빡임이 발생했다.
    /// currentKey의 status가 .loading이면 ShimmerView를 표시해 빈 상태 노출을 막는다.
    var isLoading: Bool {
        phase == .initialLoading || tagStates[currentKey]?.status == .loading
    }
    var isRefreshing: Bool { phase == .refreshing }

    // MARK: - Error

    private(set) var errorMessage: String?

    // MARK: - TagState Store

    /// 탭별 페이지네이션 상태. 키: tagId?.uuidString ?? "all"
    private var tagStates: [String: TagState] = [:]

    // MARK: - Derived Feed Items

    /// 현재 탭 아이템 목록. tagStates에서 투영.
    var feedItems: [FeedItem] {
        tagStates[currentKey]?.items ?? []
    }

    /// 현재 탭에 더 불러올 페이지가 있는지.
    var hasMore: Bool {
        tagStates[currentKey]?.hasMore ?? true
    }

    /// 현재 탭의 추가 로딩 중 여부 (sentinel ProgressView 표시용).
    var isLoadingMore: Bool {
        tagStates[currentKey]?.status == .loadingMore
    }

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
        case .loadMore:
            await loadMore()
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

    private var currentKey: String {
        cacheKey(for: selectedTagId)
    }

    private func cacheKey(for tagId: UUID?) -> String {
        tagId?.uuidString ?? "all"
    }

    /// items를 tagId 기준으로 그룹핑해 tagStates를 완전 재구성.
    /// "all" 탭 + 각 tagId 탭 포함. 태그 탭은 hasMore=false 고정 (F-08).
    /// loadInitial/refresh 공통 — 항상 완전 재구성해 stale 캐시 제거.
    /// items를 tagId 기준으로 그룹핑해 tagStates를 완전 재구성.
    /// "all" 탭 + 각 tagId 탭 포함. 태그 탭은 hasMore=false 고정 (F-08).
    /// loadInitial/refresh 공통 — 항상 완전 재구성해 stale 캐시 제거.
    ///
    /// Trade-off (F-08): selectTag 캐시 미스 폴백으로 fetch된 풍부한 태그 캐시도
    /// refresh 시 all-feed 첫 페이지 분할분으로 축소됨 (의도적, 전체 재요청 정책).
    private func rebuildTagStates(from items: [FeedItem]) {
        // 'all' 탭 + tagId별 새 캐시 구성 후 교체 — 이전 캐시 완전 제거 (stale 방지)
        var newStates: [String: TagState] = ["all": .firstPage(items: items, pageSize: PAGE_SIZE)]
        let grouped = Dictionary(grouping: items) { $0.tagId }
        for (tagId, group) in grouped {
            if let tid = tagId {
                newStates[tid.uuidString] = TagState(
                    items: group,
                    nextOffset: group.count,
                    hasMore: false,
                    status: .idle
                )
            }
        }
        tagStates = newStates
    }

    // MARK: - Core Logic

    private func loadInitial() async {
        guard !hasLoadedInitially else { return }
        hasLoadedInitially = true
        beginLoading()
        do {
            // MVP11 M4 perf: fetchAllTags + fetchMyTagIds 병렬 호출
            async let allTagsTask = tag.fetchAllTags()
            async let myTagIdsTask = tag.fetchMyTagIds()
            let allTags = try await allTagsTask
            let myTagIds = try await myTagIdsTask
            tags = allTags.filter { myTagIds.contains($0.id) }

            // ST5 C안: 전체 탭 첫 페이지만 로드 (구독 태그 프리패치 제거)
            let items = try await article.fetchFeed(tagId: nil, noCache: false, limit: PAGE_SIZE, offset: 0)
            // MVP13 M2: tagId 기준 완전 재구성 — 탭 전환 즉시 표시 (hasMore=false 고정)
            rebuildTagStates(from: items)

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
        if tagStates[key] != nil {
            selectedTagId = tagId
            errorMessage = nil
            return
        }

        // 캐시 미스 → 첫 페이지 조용히 fetch (status = .loading, 로딩 표시 없음)
        let previousTagId = selectedTagId
        var state = TagState()
        state.status = .loading
        tagStates[key] = state
        selectedTagId = tagId

        do {
            let items = try await article.fetchFeed(tagId: tagId, noCache: false, limit: PAGE_SIZE, offset: 0)
            tagStates[key] = .firstPage(items: items, pageSize: PAGE_SIZE)
            // 이전 탭 전환 에러가 남아 있을 수 있으므로 성공 시 클리어
            errorMessage = nil
        } catch {
            // 에러 시 캐시를 제거해 다음 탭 전환 시 재시도 가능하게 함.
            // selectedTagId 롤백은 현재 탭이 여전히 실패한 탭일 때만 수행
            // — 동시 탭 전환으로 다른 탭이 선택된 경우 덮어쓰기 방지.
            tagStates.removeValue(forKey: key)
            if selectedTagId == tagId {
                selectedTagId = previousTagId
            }
            errorMessage = "태그 피드를 불러오지 못했습니다."
        }
    }

    private func refresh() async {
        guard phase == .idle else { return }
        beginRefresh()

        do {
            // MVP10 M3: pull-to-refresh는 서버 TTL 캐시 우회. 첫 페이지만 다시 로드.
            // MVP13 M2: 항상 전체 재요청 후 rebuildTagStates로 완전 재구성.
            // stale 태그 캐시(이전에 있었지만 새 fetch에 없는 태그) 자동 제거.
            let items = try await article.fetchFeed(tagId: nil, noCache: true, limit: PAGE_SIZE, offset: 0)
            rebuildTagStates(from: items)

            // 현재 선택 탭이 전체 재구성 후에도 없으면 별도 fetch (selectedTagId 리셋 금지).
            // 구독 태그가 많아 첫 20개에 현재 태그 기사가 포함되지 않을 때 발생.
            if let tagId = selectedTagId, tagStates[cacheKey(for: tagId)] == nil {
                let tagItems = try await article.fetchFeed(tagId: tagId, noCache: true, limit: PAGE_SIZE, offset: 0)
                tagStates[cacheKey(for: tagId)] = .firstPage(items: tagItems, pageSize: PAGE_SIZE)
            }

            finishRefresh()
        } catch {
            failRefresh("새로고침에 실패했습니다.")
        }
    }

    private func reloadAfterTagChange() async {
        selectedTagId = nil
        tagStates = [:]
        hasLoadedInitially = false
        await loadInitial()
    }

    /// ST5: 현재 탭의 다음 페이지를 로드해 items에 누적.
    private func loadMore() async {
        let key = currentKey
        guard var state = tagStates[key],
              state.hasMore,
              state.status == .idle else { return }

        state.status = .loadingMore
        tagStates[key] = state

        do {
            let items = try await article.fetchFeed(
                tagId: selectedTagId,
                noCache: false,
                limit: PAGE_SIZE,
                offset: state.nextOffset
            )
            var updated = tagStates[key] ?? TagState()
            updated.items += items
            updated.nextOffset += items.count
            updated.hasMore = items.count == PAGE_SIZE
            updated.status = .idle
            tagStates[key] = updated
        } catch {
            // 에러 시 hasMore = false로 sentinel 비활성화 (무한 재시도 방지).
            // pull-to-refresh로 복구 가능.
            if var updated = tagStates[key] {
                updated.status = .idle
                updated.hasMore = false
                tagStates[key] = updated
            }
        }
    }
}
