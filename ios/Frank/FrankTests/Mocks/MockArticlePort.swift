import Foundation
@testable import Frank

final class MockArticlePort: ArticlePort, @unchecked Sendable {
    var articles: [Article] = []
    var articleDetail: Article?
    var fetchError: Error?

    var fetchArticlesCallCount = 0
    var fetchArticleCallCount = 0

    func fetchArticles(limit: Int) async throws -> [Article] {
        fetchArticlesCallCount += 1
        if let error = fetchError { throw error }
        return Array(articles.prefix(limit))
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
