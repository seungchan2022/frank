import Testing
import Foundation
@testable import Frank

@MainActor
@Suite("Router")
struct RouterTests {
    @Test("초기 상태는 빈 경로")
    func initialState() {
        let router = Router()
        #expect(router.path.count == 0)
    }

    @Test("navigate로 경로 추가")
    func navigatePushes() {
        let router = Router()

        router.navigate(to: .feed)

        #expect(router.path.count == 1)
    }

    @Test("pop으로 마지막 경로 제거")
    func popRemovesLast() {
        let router = Router()
        router.navigate(to: .feed)
        router.navigate(to: .settings)

        router.pop()

        #expect(router.path.count == 1)
    }

    @Test("빈 경로에서 pop은 안전")
    func popOnEmptyIsSafe() {
        let router = Router()

        router.pop()

        #expect(router.path.count == 0)
    }

    @Test("popToRoot로 전체 초기화")
    func popToRootClears() {
        let router = Router()
        router.navigate(to: .feed)
        router.navigate(to: .articleDetail(id: UUID()))
        router.navigate(to: .settings)

        router.popToRoot()

        #expect(router.path.count == 0)
    }
}
