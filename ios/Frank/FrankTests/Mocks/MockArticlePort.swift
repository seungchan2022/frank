import Foundation
@testable import Frank

/// MVP5 M1: ArticlePort Mock — fetchFeed(tagId:) 기반.
/// MVP6 M3: tagId 있으면 해당 태그 아이템만 반환 (서버 동작 시뮬레이션).
@MainActor
final class MockArticlePort: ArticlePort, @unchecked Sendable {
    var feedItems: [FeedItem] = []
    var fetchError: Error?
    var fetchFeedCallCount = 0
    /// 마지막으로 전달된 tagId 기록 (검증용)
    var lastFetchTagId: UUID?? = .none
    /// 마지막으로 전달된 noCache 값 (검증용)
    var lastFetchNoCache: Bool = false

    var lastFetchLimit: Int? = nil
    var lastFetchOffset: Int? = nil

    func fetchFeed(tagId: UUID?, noCache: Bool = false, limit: Int? = nil, offset: Int? = nil) async throws -> [FeedItem] {
        fetchFeedCallCount += 1
        lastFetchTagId = tagId
        lastFetchNoCache = noCache
        lastFetchLimit = limit
        lastFetchOffset = offset
        if let error = fetchError { throw error }
        let filtered: [FeedItem]
        if let tagId {
            filtered = feedItems.filter { $0.tagId == tagId }
        } else {
            filtered = feedItems
        }
        // limit/offset 적용
        let start = offset ?? 0
        guard start < filtered.count else { return [] }
        if let limit {
            return Array(filtered[start..<min(start + limit, filtered.count)])
        }
        return Array(filtered[start...])
    }
}
