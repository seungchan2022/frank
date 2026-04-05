import Foundation

protocol ArticlePort: Sendable {
    func fetchArticles(limit: Int) async throws -> [Article]
    func fetchArticle(id: UUID) async throws -> Article
}
