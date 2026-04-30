import Foundation

/// In-memory ArticlePort 구현. fixture 기반.
/// MVP5 M1: fetchFeed() 반환 — DB 저장 없는 ephemeral 피드.
///
/// I-04 2단계 지원: `FRANK_UI_SCENARIO=feed_refresh_2step` 환경변수 설정 시
/// noCache=true(pull-to-refresh) 호출에서 다른 fixture 세트를 반환하여
/// 새로고침 후 목록 변경을 시뮬레이션한다.
actor MockArticleAdapter: ArticlePort {
    private var feedItems: [FeedItem]
    private let refreshItems: [FeedItem]?
    private var noCacheCallCount = 0

    init(seed: [FeedItem] = MockFixtures.feedItems, refreshSeed: [FeedItem]? = nil) {
        self.feedItems = seed
        self.refreshItems = refreshSeed
    }

    func fetchFeed(tagId: UUID?, noCache: Bool = false, limit: Int? = nil, offset: Int? = nil) async throws -> [FeedItem] {
        // I-04 2단계: pull-to-refresh(noCache=true) 시 별도 fixture 반환
        let source: [FeedItem]
        if noCache, let refreshItems {
            noCacheCallCount += 1
            source = refreshItems
        } else {
            source = feedItems
        }

        // tagId 있으면 해당 태그 아이템만 반환 (서버 동작 시뮬레이션)
        let filtered: [FeedItem]
        if let tagId {
            filtered = source.filter { $0.tagId == tagId }
        } else {
            filtered = source
        }
        // limit/offset 적용 (무한 스크롤 시뮬레이션)
        let start = offset ?? 0
        guard start < filtered.count else { return [] }
        if let limit {
            return Array(filtered[start..<min(start + limit, filtered.count)])
        }
        return Array(filtered[start...])
    }
}

enum MockAdapterError: LocalizedError {
    case notFound
    case unauthorized

    var errorDescription: String? {
        switch self {
        case .notFound: "Mock: not found"
        case .unauthorized: "Mock: unauthorized"
        }
    }
}
