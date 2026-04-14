import Foundation
import Supabase

@MainActor
final class AppDependencies {
    let auth: any AuthPort
    let tag: any TagPort
    let article: any ArticlePort
    let summarize: any SummarizePort
    let favorites: any FavoritesPort
    let likes: any LikesPort
    /// MVP7 M3: 연관 기사 조회 포트
    let related: any RelatedPort
    /// MVP8 M2: 퀴즈 포트 — ArticleDetailView에 직접 주입 (Mock 기본값 제거 버그 수정)
    let quiz: any QuizPort
    /// MVP8 M3: 오답 아카이빙 포트
    let wrongAnswer: any WrongAnswerPort

    init(
        auth: any AuthPort,
        tag: any TagPort,
        article: any ArticlePort,
        summarize: any SummarizePort,
        favorites: any FavoritesPort,
        likes: any LikesPort,
        related: any RelatedPort,
        quiz: any QuizPort,
        wrongAnswer: any WrongAnswerPort
    ) {
        self.auth = auth
        self.tag = tag
        self.article = article
        self.summarize = summarize
        self.favorites = favorites
        self.likes = likes
        self.related = related
        self.quiz = quiz
        self.wrongAnswer = wrongAnswer
    }

    static func live() -> AppDependencies {
        // FRANK_USE_MOCK=1 환경변수 설정 시 Mock 어댑터 사용 (병렬 개발/UI 테스트 격리)
        // Xcode scheme: Edit Scheme → Run → Arguments → Environment Variables
        if ProcessInfo.processInfo.environment["FRANK_USE_MOCK"] == "1" {
            return mock()
        }

        // MVP5 M1: 인증은 Supabase SDK, 데이터는 Rust API 어댑터.
        // CollectPort 제거 — 피드는 GET /api/me/feed 직접 호출.
        let config = SupabaseConfig.live
        let serverConfig = ServerConfig.live
        let client = SupabaseClient(supabaseURL: config.url, supabaseKey: config.anonKey)
        let authAdapter = SupabaseAuthAdapter(
            client: client,
            serverConfig: serverConfig
        )

        // 요약 요청 전용 URLSession — 일반 요청과 세션 분리 (테스트 주입 지원)
        // 실제 타임아웃은 APISummarizeAdapter 내부 URLRequest.timeoutInterval(70초)이 결정
        let summarizeSession = URLSession(configuration: .default)

        return AppDependencies(
            auth: authAdapter,
            tag: APITagAdapter(auth: authAdapter, serverConfig: serverConfig),
            article: APIArticleAdapter(auth: authAdapter, serverConfig: serverConfig),
            summarize: APISummarizeAdapter(
                auth: authAdapter,
                serverConfig: serverConfig,
                session: summarizeSession
            ),
            favorites: APIFavoritesAdapter(auth: authAdapter, serverConfig: serverConfig),
            likes: APILikesAdapter(auth: authAdapter, serverConfig: serverConfig),
            related: APIRelatedAdapter(auth: authAdapter, serverConfig: serverConfig),
            quiz: APIQuizAdapter(auth: authAdapter, serverConfig: serverConfig),
            wrongAnswer: APIWrongAnswerAdapter(auth: authAdapter, serverConfig: serverConfig)
        )
    }

    /// Mock 의존성 — fixture 기반, 외부 호출 0.
    ///
    /// `FRANK_UI_SCENARIO` 환경변수로 시나리오 주입:
    /// - `logged_out`: 로그인 화면 노출 (TC-01)
    /// - `new_user`: 온보딩 화면 노출 (TC-02)
    static func mock() -> AppDependencies {
        let scenario = ProcessInfo.processInfo.environment["FRANK_UI_SCENARIO"]

        let profile = scenario == "new_user" ? MockFixtures.newUserProfile : MockFixtures.profile

        return AppDependencies(
            auth: MockAuthAdapter(profile: profile, scenario: scenario),
            tag: MockTagAdapter(),
            article: MockArticleAdapter(),
            summarize: MockSummarizeAdapter(),
            favorites: MockFavoritesAdapter(),
            likes: MockLikesAdapter(),
            related: MockRelatedAdapter(),
            quiz: MockQuizAdapter(),
            wrongAnswer: MockWrongAnswerAdapter()
        )
    }
}

// MARK: - Placeholder Ports (참고용 — 미사용)

private struct PlaceholderTagPort: TagPort {
    func fetchAllTags() async throws -> [Tag] { [] }
    func fetchMyTagIds() async throws -> [UUID] { [] }
    func saveMyTags(tagIds: [UUID]) async throws {}
}

private struct PlaceholderArticlePort: ArticlePort {
    func fetchFeed(tagId: UUID?, noCache: Bool) async throws -> [FeedItem] { [] }
}
