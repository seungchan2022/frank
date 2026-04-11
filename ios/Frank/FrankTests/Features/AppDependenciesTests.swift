import Testing
@testable import Frank

@MainActor
@Suite("AppDependencies")
struct AppDependenciesTests {
    @Test("DI 컨테이너가 모든 포트를 보유한다")
    func containerHoldsAllPorts() {
        let deps = AppDependencies(
            auth: MockAuthPort(),
            tag: MockTagPort(),
            article: MockArticlePort(),
            summarize: MockSummarizePort(),
            favorites: MockFavoritesPort(),
            likes: MockLikesPort()
        )

        #expect(deps.auth is MockAuthPort)
        #expect(deps.tag is MockTagPort)
        #expect(deps.article is MockArticlePort)
        #expect(deps.summarize is MockSummarizePort)
        #expect(deps.favorites is MockFavoritesPort)
        #expect(deps.likes is MockLikesPort)
    }
}
