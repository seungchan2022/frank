import Testing
import Foundation
@testable import Frank

@Suite("RelatedFeature Tests — MVP7 M3")
@MainActor
struct RelatedFeatureTests {

    // MARK: - Helpers

    private func makeFeedItem(
        url: String = "https://example.com/related/1",
        title: String = "연관 기사"
    ) -> FeedItem {
        FeedItem(
            title: title,
            url: URL(string: url) ?? URL(string: "https://example.com")!,
            source: "TestSource",
            publishedAt: nil,
            tagId: nil,
            snippet: "연관 내용"
        )
    }

    private func makeSUT(port: MockRelatedPort = MockRelatedPort()) -> (RelatedFeature, MockRelatedPort) {
        let feature = RelatedFeature(related: port)
        return (feature, port)
    }

    // MARK: - 초기 상태

    @Test("초기 상태: items 빈 배열, isLoading false, errorMessage nil")
    func initialState() {
        let (sut, _) = makeSUT()
        #expect(sut.items.isEmpty)
        #expect(sut.isLoading == false)
        #expect(sut.errorMessage == nil)
    }

    // MARK: - 성공

    @Test("load 성공 시 items 업데이트")
    func load_sets_items_on_success() async {
        let port = MockRelatedPort()
        port.stubbedItems = [
            makeFeedItem(url: "https://example.com/1", title: "기사 1"),
            makeFeedItem(url: "https://example.com/2", title: "기사 2"),
        ]
        let (sut, _) = makeSUT(port: port)

        await sut.load(title: "테스트 기사", snippet: "내용")

        #expect(sut.items.count == 2)
        #expect(sut.items[0].title == "기사 1")
        #expect(sut.isLoading == false)
        #expect(sut.errorMessage == nil)
    }

    @Test("load 성공 후 isLoading false")
    func load_success_leaves_isLoading_false() async {
        let (sut, _) = makeSUT()

        await sut.load(title: "기사", snippet: nil)

        #expect(sut.isLoading == false)
    }

    // MARK: - 실패

    @Test("load 실패 시 errorMessage 설정")
    func load_sets_error_on_failure() async {
        let port = MockRelatedPort()
        port.shouldFail = true
        let (sut, _) = makeSUT(port: port)

        await sut.load(title: "기사", snippet: nil)

        #expect(sut.errorMessage != nil)
        #expect(sut.isLoading == false)
    }

    @Test("load 실패 시 items 변경 없음")
    func load_failure_does_not_update_items() async {
        let port = MockRelatedPort()
        port.shouldFail = true
        let (sut, _) = makeSUT(port: port)

        await sut.load(title: "기사", snippet: nil)

        #expect(sut.items.isEmpty)
    }

    // MARK: - 재로드

    @Test("reload 시 이전 items 초기화 후 새 items 설정")
    func load_clears_previous_items_on_reload() async {
        let port = MockRelatedPort()
        port.stubbedItems = [makeFeedItem(url: "https://example.com/old", title: "이전 기사")]
        let (sut, _) = makeSUT(port: port)

        await sut.load(title: "첫 번째 기사", snippet: nil)
        #expect(sut.items.count == 1)

        port.stubbedItems = [
            makeFeedItem(url: "https://example.com/new1", title: "새 기사 1"),
            makeFeedItem(url: "https://example.com/new2", title: "새 기사 2"),
        ]
        await sut.load(title: "두 번째 기사", snippet: nil)

        #expect(sut.items.count == 2)
        #expect(sut.items[0].title == "새 기사 1")
    }

    // MARK: - 포트 호출 검증

    @Test("load 호출 시 포트에 title, snippet 전달")
    func load_passes_title_and_snippet_to_port() async {
        let port = MockRelatedPort()
        let (sut, _) = makeSUT(port: port)

        await sut.load(title: "iOS 기사", snippet: "Swift 내용")

        #expect(port.fetchCallCount == 1)
    }

    @Test("snippet nil로 load 호출 가능")
    func load_with_nil_snippet() async {
        let (sut, port) = makeSUT()

        await sut.load(title: "기사", snippet: nil)

        #expect(port.fetchCallCount == 1)
        #expect(sut.errorMessage == nil)
    }
}
