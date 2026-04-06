import Foundation
@testable import Frank

final class MockArticlePort: ArticlePort, @unchecked Sendable {
    var articles: [Article] = []
    var articleDetail: Article?
    var fetchError: Error?
    var articlesByTag: [UUID: [Article]] = [:]
    var fetchSequence: [[Article]] = []

    var fetchArticlesCallCount = 0
    var fetchArticlesFilteredCallCount = 0
    var fetchArticleCallCount = 0
    var lastFilter: ArticleFilter?

    private var fetchSequenceIndex = 0

    func fetchArticles(filter: ArticleFilter) async throws -> [Article] {
        fetchArticlesCallCount += 1
        fetchArticlesFilteredCallCount += 1
        lastFilter = filter
        if let error = fetchError { throw error }

        // fetchSequence가 있으면 순서대로 반환
        if !fetchSequence.isEmpty {
            let result = fetchSequence[min(fetchSequenceIndex, fetchSequence.count - 1)]
            fetchSequenceIndex += 1
            return result
        }

        // tagId 필터링
        var filtered = articles
        if let tagId = filter.tagId {
            if let tagArticles = articlesByTag[tagId] {
                filtered = tagArticles
            } else {
                filtered = articles.filter { $0.tagId == tagId }
            }
        }

        // offset + limit
        let start = min(filter.offset, filtered.count)
        let end = min(start + filter.limit, filtered.count)
        return Array(filtered[start..<end])
    }

    func fetchArticle(id: UUID) async throws -> Article {
        fetchArticleCallCount += 1
        if let error = fetchError { throw error }
        guard let article = articleDetail ?? articles.first(where: { $0.id == id }) else {
            throw URLError(.badServerResponse)
        }
        return article
    }
}
