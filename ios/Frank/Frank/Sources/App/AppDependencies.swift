import Foundation
import Supabase

@MainActor
final class AppDependencies {
    let auth: any AuthPort
    let tag: any TagPort
    let article: any ArticlePort
    let collect: any CollectPort

    init(
        auth: any AuthPort,
        tag: any TagPort,
        article: any ArticlePort,
        collect: any CollectPort
    ) {
        self.auth = auth
        self.tag = tag
        self.article = article
        self.collect = collect
    }

    static func live() -> AppDependencies {
        let config = SupabaseConfig.live
        let client = SupabaseClient(supabaseURL: config.url, supabaseKey: config.anonKey)
        let authAdapter = SupabaseAuthAdapter(client: client)

        return AppDependencies(
            auth: authAdapter,
            tag: SupabaseTagAdapter(client: client),
            article: SupabaseArticleAdapter(client: client),
            collect: APICollectAdapter(
                auth: authAdapter,
                serverConfig: ServerConfig.live
            )
        )
    }
}

// MARK: - Placeholder Ports (M3~M6에서 프로덕션 어댑터로 교체)

private struct PlaceholderTagPort: TagPort {
    func fetchAllTags() async throws -> [Tag] { [] }
    func fetchMyTagIds() async throws -> [UUID] { [] }
    func saveMyTags(tagIds: [UUID]) async throws {}
}

private struct PlaceholderArticlePort: ArticlePort {
    func fetchArticles(filter: ArticleFilter) async throws -> [Article] { [] }
    func fetchArticle(id: UUID) async throws -> Article {
        throw URLError(.resourceUnavailable)
    }
}

private struct PlaceholderCollectPort: CollectPort {
    func triggerCollect() async throws -> Int { 0 }
    func triggerSummarize() async throws -> Int { 0 }
}
