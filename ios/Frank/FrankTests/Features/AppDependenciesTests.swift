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

    @Test("summarizeTimeoutSeconds 기본값은 30초다")
    func summarizeTimeoutDefaultIs30() {
        let deps = AppDependencies(
            auth: MockAuthPort(),
            tag: MockTagPort(),
            article: MockArticlePort(),
            collect: MockCollectPort()
        )
        #expect(deps.summarizeTimeoutSeconds == 30)
    }

    @Test("summarizeTimeoutSeconds 주입값이 반영된다")
    func summarizeTimeoutInjected() {
        let deps = AppDependencies(
            auth: MockAuthPort(),
            tag: MockTagPort(),
            article: MockArticlePort(),
            collect: MockCollectPort(),
            summarizeTimeoutSeconds: 5
        )
        #expect(deps.summarizeTimeoutSeconds == 5)
    }
}
