import Foundation
@testable import Frank

/// MVP5 M1: ArticlePort Mock — fetchFeed() 기반.
final class MockArticlePort: ArticlePort, @unchecked Sendable {
    var feedItems: [FeedItem] = []
    var fetchError: Error?
    var fetchFeedCallCount = 0

    func fetchFeed() async throws -> [FeedItem] {
        fetchFeedCallCount += 1
        if let error = fetchError { throw error }
        return feedItems
    }
}
