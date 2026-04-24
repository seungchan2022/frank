import Testing
@testable import Frank

@Suite("AppDependencies")
@MainActor
struct AppDependenciesTests {
    @Test("DI 컨테이너가 모든 포트를 보유한다")
    func containerHoldsAllPorts() {
        let deps = AppDependencies(
            auth: MockAuthPort(),
            tag: MockTagPort(),
            article: MockArticlePort(),
            summarize: MockSummarizePort(),
            favorites: MockFavoritesPort(),
            likes: MockLikesPort(),
            related: MockRelatedPort(),
            quiz: MockQuizPort(),
            wrongAnswer: MockWrongAnswerPort()
        )

        #expect(deps.auth is MockAuthPort)
        #expect(deps.tag is MockTagPort)
        #expect(deps.article is MockArticlePort)
        #expect(deps.summarize is MockSummarizePort)
        #expect(deps.favorites is MockFavoritesPort)
        #expect(deps.likes is MockLikesPort)
        #expect(deps.related is MockRelatedPort)
        #expect(deps.quiz is MockQuizPort)
        #expect(deps.wrongAnswer is MockWrongAnswerPort)
    }

    // MARK: - Bootstrap

    @Test("mock()은 MockAuthAdapter를 포함한 모든 Mock 어댑터를 반환한다")
    func mockReturnsAllMockAdapters() {
        let deps = AppDependencies.mock()
        #expect(deps.auth is MockAuthAdapter)
        #expect(deps.tag is MockTagAdapter)
        #expect(deps.article is MockArticleAdapter)
        #expect(deps.summarize is MockSummarizeAdapter)
        #expect(deps.favorites is MockFavoritesAdapter)
        #expect(deps.likes is MockLikesAdapter)
        #expect(deps.related is MockRelatedAdapter)
        #expect(deps.quiz is MockQuizAdapter)
        #expect(deps.wrongAnswer is MockWrongAnswerAdapter)
    }

    @Test("시뮬레이터에서 bootstrap()은 .ready 반환 (ServerConfig.live() → localhost:8080)")
    func bootstrapReturnsReadyInSimulator() {
        #if targetEnvironment(simulator)
        let result = AppDependencies.bootstrap()
        if case .configError(let error) = result {
            Issue.record("시뮬레이터에서 configError 발생: \(error)")
        }
        #else
        #expect(Bool(true))
        #endif
    }
}
