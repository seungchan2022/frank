import Testing
import Foundation
@testable import Frank

@Suite("LikesFeature Tests — MVP7 M2")
@MainActor
struct LikesFeatureTests {

    // MARK: - Helpers

    private func makeFeedItem(url: String = "https://example.com/article") -> FeedItem {
        FeedItem(
            title: "iOS 기사",
            url: URL(string: url)!,
            source: "TestSource",
            publishedAt: nil,
            tagId: nil,
            snippet: "Swift 내용"
        )
    }

    private func makeSUT(port: MockLikesPort = MockLikesPort()) -> (LikesFeature, MockLikesPort) {
        let feature = LikesFeature(likes: port)
        return (feature, port)
    }

    // MARK: - 초기 상태

    @Test("초기 상태: likedUrls는 빈 Set")
    func initialState() {
        let (sut, _) = makeSUT()
        #expect(sut.likedUrls.isEmpty)
        #expect(!sut.isLiked("https://example.com"))
    }

    // MARK: - like 성공

    @Test("like 성공 시 likedUrls에 url 추가")
    func likeSuccessAddsUrl() async {
        let (sut, port) = makeSUT()
        let item = makeFeedItem()

        await sut.like(feedItem: item)

        #expect(sut.isLiked(item.url.absoluteString))
        #expect(port.likeCallCount == 1)
    }

    @Test("like 성공 시 lastKeywords 업데이트")
    func likeSuccessUpdatesKeywords() async {
        let port = MockLikesPort()
        port.stubbedKeywords = ["iOS", "Swift", "SwiftUI"]
        let (sut, _) = makeSUT(port: port)
        let item = makeFeedItem()

        await sut.like(feedItem: item)

        #expect(sut.lastKeywords == ["iOS", "Swift", "SwiftUI"])
    }

    // MARK: - 중복 방지

    @Test("같은 url 두 번 like 시 API 1회만 호출")
    func likeDeduplicates() async {
        let (sut, port) = makeSUT()
        let item = makeFeedItem()

        await sut.like(feedItem: item)
        await sut.like(feedItem: item) // 중복

        #expect(port.likeCallCount == 1)
        #expect(sut.isLiked(item.url.absoluteString))
    }

    // MARK: - like 실패

    @Test("like 실패 시 likedUrls 변경 없음")
    func likeFailureDoesNotAddUrl() async {
        let port = MockLikesPort()
        port.shouldFail = true
        let (sut, _) = makeSUT(port: port)
        let item = makeFeedItem()

        await sut.like(feedItem: item)

        #expect(!sut.isLiked(item.url.absoluteString))
    }

    @Test("like 실패 시 error 설정")
    func likeFailureSetsError() async {
        let port = MockLikesPort()
        port.shouldFail = true
        let (sut, _) = makeSUT(port: port)
        let item = makeFeedItem()

        await sut.like(feedItem: item)

        #expect(sut.error != nil)
    }

    // MARK: - isLiked

    @Test("isLiked: 없는 url은 false")
    func isLikedReturnsFalseForUnknown() {
        let (sut, _) = makeSUT()
        #expect(!sut.isLiked("https://unknown.com"))
    }

    // MARK: - 여러 기사 누적

    @Test("여러 기사 like 누적")
    func multipleArticlesAccumulate() async {
        let (sut, port) = makeSUT()
        let item1 = makeFeedItem(url: "https://example.com/1")
        let item2 = makeFeedItem(url: "https://example.com/2")

        port.stubbedTotalLikes = 1
        await sut.like(feedItem: item1)
        port.stubbedTotalLikes = 2
        await sut.like(feedItem: item2)

        #expect(sut.likedUrls.count == 2)
        #expect(sut.isLiked("https://example.com/1"))
        #expect(sut.isLiked("https://example.com/2"))
    }
}
