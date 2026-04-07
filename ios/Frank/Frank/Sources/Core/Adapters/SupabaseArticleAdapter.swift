import Foundation
import Supabase

struct SupabaseArticleAdapter: ArticlePort {
    private let client: SupabaseClient

    init(client: SupabaseClient) {
        self.client = client
    }

    func fetchArticles(filter: ArticleFilter) async throws -> [Article] {
        var query = client.from("articles")
            .select()

        if let tagId = filter.tagId {
            query = query.eq("tag_id", value: tagId)
        }

        let response: [ArticleDTO] = try await query
            .order("created_at", ascending: false)
            .range(from: filter.offset, to: filter.offset + filter.limit - 1)
            .execute()
            .value

        return response.map { $0.toDomain() }
    }

    func fetchArticle(id: UUID) async throws -> Article {
        let response: ArticleDTO = try await client.from("articles")
            .select()
            .eq("id", value: id)
            .single()
            .execute()
            .value

        return response.toDomain()
    }
}

// MARK: - DTOs

private enum ArticleDTOConstants {
    // Known-valid literal — safe to force-unwrap at compile time
    // swiftlint:disable:next force_unwrapping
    static let fallbackURL = URL(string: "https://example.com")!
}

private struct ArticleDTO: Decodable {
    let id: UUID
    let userId: UUID
    let title: String
    let url: String
    let source: String
    let publishedAt: Date?
    let summary: String?
    let tagId: UUID?
    let titleKo: String?
    let insight: String?
    let snippet: String?
    let summarizedAt: Date?
    let createdAt: Date?
    let searchQuery: String?

    enum CodingKeys: String, CodingKey {
        case id, title, url, source, summary, insight, snippet
        case userId = "user_id"
        case publishedAt = "published_at"
        case tagId = "tag_id"
        case titleKo = "title_ko"
        case summarizedAt = "summarized_at"
        case createdAt = "created_at"
        case searchQuery = "search_query"
    }

    func toDomain() -> Article {
        Article(
            id: id,
            userId: userId,
            title: title,
            url: URL(string: url) ?? ArticleDTOConstants.fallbackURL,
            source: source,
            publishedAt: publishedAt,
            summary: summary,
            tagId: tagId,
            titleKo: titleKo,
            insight: insight,
            snippet: snippet,
            summarizedAt: summarizedAt,
            searchQuery: searchQuery,
            createdAt: createdAt
        )
    }
}
