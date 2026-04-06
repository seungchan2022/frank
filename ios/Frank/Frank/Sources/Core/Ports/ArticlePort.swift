import Foundation

protocol ArticlePort: Sendable {
    func fetchArticles(filter: ArticleFilter) async throws -> [Article]
    func fetchArticle(id: UUID) async throws -> Article
}
