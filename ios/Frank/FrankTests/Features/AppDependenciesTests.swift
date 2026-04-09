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
            collect: MockCollectPort()
        )

        #expect(deps.auth is MockAuthPort)
        #expect(deps.tag is MockTagPort)
        #expect(deps.article is MockArticlePort)
        #expect(deps.collect is MockCollectPort)
    }
}
