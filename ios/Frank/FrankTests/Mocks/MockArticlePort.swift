import Foundation
@testable import Frank

/// MVP5 M1: ArticlePort Mock — fetchFeed(tagId:) 기반.
/// MVP6 M3: tagId 있으면 해당 태그 아이템만 반환 (서버 동작 시뮬레이션).
final class MockArticlePort: ArticlePort, @unchecked Sendable {
    var feedItems: [FeedItem] = []
    var fetchError: Error?
    var fetchFeedCallCount = 0
    /// 마지막으로 전달된 tagId 기록 (검증용)
    var lastFetchTagId: UUID?? = .none

    func fetchFeed(tagId: UUID?) async throws -> [FeedItem] {
        fetchFeedCallCount += 1
        lastFetchTagId = tagId
        if let error = fetchError { throw error }
        guard let tagId else { return feedItems }
        return feedItems.filter { $0.tagId == tagId }
    }
}
