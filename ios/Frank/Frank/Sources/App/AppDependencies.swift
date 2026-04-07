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
        // FRANK_USE_MOCK=1 환경변수 설정 시 Mock 어댑터 사용 (병렬 개발/UI 테스트 격리)
        // Xcode scheme: Edit Scheme → Run → Arguments → Environment Variables
        if ProcessInfo.processInfo.environment["FRANK_USE_MOCK"] == "1" {
            return mock()
        }

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

    /// Mock 의존성 — fixture 기반, 외부 호출 0.
    /// M1.5 병렬 개발 시 양쪽 탭이 외부 자원을 공유하지 않도록 격리.
    static func mock() -> AppDependencies {
        AppDependencies(
            auth: MockAuthAdapter(),
            tag: MockTagAdapter(),
            article: MockArticleAdapter(),
            collect: MockCollectAdapter()
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
