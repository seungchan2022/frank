import Foundation

/// In-memory ArticlePort 구현. fixture 기반.
/// MVP5 M1: fetchFeed() 반환 — DB 저장 없는 ephemeral 피드.
actor MockArticleAdapter: ArticlePort {
    private var feedItems: [FeedItem]

    init(seed: [FeedItem] = MockFixtures.feedItems) {
        self.feedItems = seed
    }

    func fetchFeed(tagId: UUID?, noCache: Bool = false, limit: Int? = nil, offset: Int? = nil) async throws -> [FeedItem] {
        // tagId 있으면 해당 태그 아이템만 반환 (서버 동작 시뮬레이션)
        let filtered: [FeedItem]
        if let tagId {
            filtered = feedItems.filter { $0.tagId == tagId }
        } else {
            filtered = feedItems
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
