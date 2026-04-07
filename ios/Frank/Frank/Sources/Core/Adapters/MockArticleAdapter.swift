import Foundation

/// In-memory ArticlePort 구현. fixture 기반.
/// M1.5 병렬 작업 시 외부 의존 격리용.
actor MockArticleAdapter: ArticlePort {
    private var articles: [Article]

    init(seed: [Article] = MockFixtures.articles) {
        self.articles = seed
    }

    func fetchArticles(filter: ArticleFilter) async throws -> [Article] {
        var filtered = articles
        if let tagId = filter.tagId {
            filtered = filtered.filter { $0.tagId == tagId }
        }
        // publishedAt desc 정렬 (nil은 가장 오래된 것으로 취급)
        filtered.sort { ($0.publishedAt ?? .distantPast) > ($1.publishedAt ?? .distantPast) }
        let start = max(0, filter.offset)
        let end = min(filtered.count, start + filter.limit)
        guard start < end else { return [] }
        return Array(filtered[start..<end])
    }

    func fetchArticle(id: UUID) async throws -> Article {
        guard let article = articles.first(where: { $0.id == id }) else {
            throw MockAdapterError.notFound
        }
        return article
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
