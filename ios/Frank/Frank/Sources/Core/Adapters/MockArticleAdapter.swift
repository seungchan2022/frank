import Foundation

/// In-memory ArticlePort 구현. fixture 기반.
/// MVP5 M1: fetchFeed() 반환 — DB 저장 없는 ephemeral 피드.
actor MockArticleAdapter: ArticlePort {
    private var feedItems: [FeedItem]

    init(seed: [FeedItem] = MockFixtures.feedItems) {
        self.feedItems = seed
    }

    func fetchFeed(tagId: UUID?) async throws -> [FeedItem] {
        // tagId 있으면 해당 태그 아이템만 반환 (서버 동작 시뮬레이션)
        guard let tagId else { return feedItems }
        return feedItems.filter { $0.tagId == tagId }
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
