import Foundation
import Supabase

@MainActor
final class AppDependencies {
    let auth: any AuthPort
    let tag: any TagPort
    let article: any ArticlePort
    let collect: any CollectPort
    /// 요약 타임아웃 (초). UITest에서 FRANK_SUMMARIZE_TIMEOUT_SECONDS로 짧게 주입 (TC-03).
    let summarizeTimeoutSeconds: Double

    init(
        auth: any AuthPort,
        tag: any TagPort,
        article: any ArticlePort,
        collect: any CollectPort,
        summarizeTimeoutSeconds: Double = 30
    ) {
        self.auth = auth
        self.tag = tag
        self.article = article
        self.collect = collect
        self.summarizeTimeoutSeconds = summarizeTimeoutSeconds
    }

    static func live() -> AppDependencies {
        // FRANK_USE_MOCK=1 환경변수 설정 시 Mock 어댑터 사용 (병렬 개발/UI 테스트 격리)
        // Xcode scheme: Edit Scheme → Run → Arguments → Environment Variables
        if ProcessInfo.processInfo.environment["FRANK_USE_MOCK"] == "1" {
            return mock()
        }

        // M3: 인증은 Supabase SDK, 데이터는 모두 Rust API 어댑터로 통합
        // - Auth: SupabaseAuthAdapter (signIn/signUp/session) + ProfileAPI(PUT /api/me/profile)
        // - Tag/Article/Collect: Rust API 어댑터
        //
        // Supabase{Tag,Article}Adapter 파일은 보존되며 사용되지 않음 (Rollback/참고용)
        let config = SupabaseConfig.live
        let serverConfig = ServerConfig.live
        let client = SupabaseClient(supabaseURL: config.url, supabaseKey: config.anonKey)
        let authAdapter = SupabaseAuthAdapter(
            client: client,
            serverConfig: serverConfig
        )

        return AppDependencies(
            auth: authAdapter,
            tag: APITagAdapter(auth: authAdapter, serverConfig: serverConfig),
            article: APIArticleAdapter(auth: authAdapter, serverConfig: serverConfig),
            collect: APICollectAdapter(auth: authAdapter, serverConfig: serverConfig),
            summarizeTimeoutSeconds: Self.parseSummarizeTimeout()
        )
    }

    /// Mock 의존성 — fixture 기반, 외부 호출 0.
    ///
    /// `FRANK_UI_SCENARIO` 환경변수로 시나리오 주입:
    /// - `logged_out`: 로그인 화면 노출 (TC-01)
    /// - `new_user`: 온보딩 화면 노출 (TC-02)
    /// - `summarize_timeout`: 요약 타임아웃 재현 (TC-03)
    static func mock() -> AppDependencies {
        let scenario = ProcessInfo.processInfo.environment["FRANK_UI_SCENARIO"]

        let profile = scenario == "new_user" ? MockFixtures.newUserProfile : MockFixtures.profile

        return AppDependencies(
            auth: MockAuthAdapter(profile: profile, scenario: scenario),
            tag: MockTagAdapter(),
            article: MockArticleAdapter(),
            collect: MockCollectAdapter(scenario: scenario),
            summarizeTimeoutSeconds: Self.parseSummarizeTimeout()
        )
    }

    // MARK: - Helpers

    /// `FRANK_SUMMARIZE_TIMEOUT_SECONDS` 환경변수를 파싱한다. 미설정/파싱 실패 시 30초 반환.
    private static func parseSummarizeTimeout() -> Double {
        Double(ProcessInfo.processInfo.environment["FRANK_SUMMARIZE_TIMEOUT_SECONDS"] ?? "") ?? 30
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
