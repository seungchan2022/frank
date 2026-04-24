import Testing
import Foundation
@testable import Frank

@Suite("FavoritesFeature Tests — M3")
@MainActor
struct FavoritesFeatureTests {

    // MARK: - Helpers

    private func makeFeedItem(url: String = "https://example.com/article", tagId: UUID? = nil) -> FeedItem {
        FeedItem(
            title: "테스트 기사",
            url: URL(string: url)!,
            source: "TestSource",
            publishedAt: nil,
            tagId: tagId,
            snippet: nil
        )
    }

    private func makeSUT(
        port: MockFavoritesPort = MockFavoritesPort(),
        tagPort: MockTagPort = MockTagPort()
    ) -> (FavoritesFeature, MockFavoritesPort, MockTagPort) {
        let feature = FavoritesFeature(favorites: port, tag: tagPort)
        return (feature, port, tagPort)
    }

    // MARK: - 1. 초기 상태

    @Test("초기 상태: phase=idle, items=빈 배열, hasLoaded=false")
    func initialState() {
        let (sut, _, _) = makeSUT()
        #expect(sut.phase == .idle)
        #expect(sut.items.isEmpty)
        #expect(sut.hasLoaded == false)
        #expect(sut.likedUrls.isEmpty)
    }

    // MARK: - 2. 목록 로딩

    @Test("loadFavorites: API 호출 → items 채워짐 + phase=done")
    func loadFavoritesSuccess() async {
        let port = MockFavoritesPort()
        let (sut, _, _) = makeSUT(port: port)

        await sut.loadFavorites()

        #expect(port.listCallCount == 1)
        #expect(sut.phase == .done)
        #expect(sut.hasLoaded == true)
    }

    @Test("loadFavorites 재호출: hasLoaded=true이면 no-op (API 1번만)")
    func loadFavoritesNoopWhenLoaded() async {
        let port = MockFavoritesPort()
        let (sut, _, _) = makeSUT(port: port)

        await sut.loadFavorites()
        await sut.loadFavorites() // 두 번째 호출 → no-op

        #expect(port.listCallCount == 1)
    }

    @Test("loadFavorites 실패: phase=failed")
    func loadFavoritesFailed() async {
        let port = MockFavoritesPort()
        port.shouldFail = true
        let (sut, _, _) = makeSUT(port: port)

        await sut.loadFavorites()

        if case .failed = sut.phase {
            // 성공
        } else {
            Issue.record("Expected failed phase, got \(sut.phase)")
        }
        #expect(sut.hasLoaded == false)
    }

    // MARK: - 3. 즐겨찾기 추가

    @Test("addFavorite: 추가 후 items에 prepend + likedUrls 업데이트")
    func addFavoriteSuccess() async {
        let (sut, _, _) = makeSUT()
        let item = makeFeedItem()

        await sut.addFavorite(feedItem: item, summary: "요약", insight: "인사이트")

        #expect(sut.items.count == 1)
        #expect(sut.items[0].url == "https://example.com/article")
        #expect(sut.isLiked("https://example.com/article") == true)
        // summary/insight가 전달됨
        #expect(sut.items[0].summary == "요약")
        #expect(sut.items[0].insight == "인사이트")
    }

    @Test("addFavorite: 중복 추가 → operationError=Conflict 메시지 (phase는 idle 유지)")
    func addFavoriteDuplicate() async {
        let port = MockFavoritesPort()
        let (sut, _, _) = makeSUT(port: port)
        let item = makeFeedItem()

        await sut.addFavorite(feedItem: item, summary: nil, insight: nil)
        port.shouldConflict = true
        await sut.addFavorite(feedItem: item, summary: nil, insight: nil)

        #expect(sut.operationError != nil)
        #expect(sut.phase == .idle)
    }

    // MARK: - 4. 즐겨찾기 삭제

    @Test("removeFavorite: 삭제 후 items에서 제거 + likedUrls 업데이트")
    func removeFavoriteSuccess() async {
        let (sut, _, _) = makeSUT()
        let item = makeFeedItem()

        await sut.addFavorite(feedItem: item, summary: nil, insight: nil)
        #expect(sut.isLiked("https://example.com/article") == true)

        await sut.removeFavorite(url: "https://example.com/article")
        #expect(sut.items.isEmpty)
        #expect(sut.isLiked("https://example.com/article") == false)
    }

    @Test("removeFavorite 실패: operationError 설정 (phase는 idle 유지, items 변경 없음)")
    func removeFavoriteFailed() async {
        let port = MockFavoritesPort()
        let (sut, _, _) = makeSUT(port: port)

        // 먼저 추가
        await sut.addFavorite(feedItem: makeFeedItem(), summary: nil, insight: nil)
        #expect(sut.items.count == 1)

        port.shouldFail = true
        await sut.removeFavorite(url: "https://example.com/article")

        #expect(sut.operationError != nil)
        #expect(sut.phase == .idle)
        #expect(sut.items.count == 1) // 삭제 실패 시 items 유지
    }

    // MARK: - 5. likedUrls

    @Test("likedUrls: items에서 url 추출한 Set")
    func likedUrlsDerived() async {
        let (sut, _, _) = makeSUT()

        await sut.addFavorite(feedItem: makeFeedItem(url: "https://a.com"), summary: nil, insight: nil)
        await sut.addFavorite(feedItem: makeFeedItem(url: "https://b.com"), summary: nil, insight: nil)

        #expect(sut.likedUrls.contains("https://a.com"))
        #expect(sut.likedUrls.contains("https://b.com"))
        #expect(sut.likedUrls.count == 2)
    }

    @Test("isLiked: 없는 url → false")
    func isLikedFalseForUnknown() {
        let (sut, _, _) = makeSUT()
        #expect(sut.isLiked("https://not-liked.com") == false)
    }

    // MARK: - 6. 퀴즈 완료 마킹 (MVP10 M1)

    @Test("markQuizCompleted 성공: items의 해당 url quizCompleted = true 로 갱신")
    func markQuizCompletedSuccess() async {
        let port = MockFavoritesPort()
        let (sut, _, _) = makeSUT(port: port)
        await sut.addFavorite(feedItem: makeFeedItem(), summary: nil, insight: nil)
        #expect(sut.isQuizCompleted("https://example.com/article") == false)

        await sut.markQuizCompleted(url: "https://example.com/article")

        #expect(port.markQuizCompletedCallCount == 1)
        #expect(sut.isQuizCompleted("https://example.com/article") == true)
        #expect(sut.operationError == nil)
    }

    @Test("markQuizCompleted 실패: operationError 설정, items 변경 없음")
    func markQuizCompletedFailed() async {
        let port = MockFavoritesPort()
        let (sut, _, _) = makeSUT(port: port)
        await sut.addFavorite(feedItem: makeFeedItem(), summary: nil, insight: nil)

        port.shouldFail = true
        await sut.markQuizCompleted(url: "https://example.com/article")

        #expect(sut.operationError != nil)
        #expect(sut.isQuizCompleted("https://example.com/article") == false)
    }

    @Test("markQuizCompleted: 없는 url → items 변경 없음")
    func markQuizCompletedUnknownUrl() async {
        let (sut, _, _) = makeSUT()
        await sut.addFavorite(feedItem: makeFeedItem(), summary: nil, insight: nil)

        await sut.markQuizCompleted(url: "https://unknown.com")

        #expect(sut.isQuizCompleted("https://example.com/article") == false)
        #expect(sut.operationError == nil)
    }

    // MARK: - 7. 태그 필터 (MVP11 M4)
    // selectedTagId는 FavoritesView @State 단일 소스 — Feature는 파라미터로 받아 계산만 (F-07 요구사항).

    @Test("filteredItems(nil): 전체 아이템 반환")
    func filteredItemsNil_returnsAll() async {
        let tagId = UUID()
        let port = MockFavoritesPort()
        let (sut, _, _) = makeSUT(port: port)

        await sut.addFavorite(feedItem: makeFeedItem(url: "https://a.com", tagId: tagId), summary: nil, insight: nil)
        await sut.addFavorite(feedItem: makeFeedItem(url: "https://b.com"), summary: nil, insight: nil)

        #expect(sut.filteredItems(selectedTagId: nil).count == 2)
    }

    @Test("filteredItems(tagId): 해당 tagId 아이템만 반환")
    func filteredItems_filtersByTagId() async {
        let tagId = UUID()
        let otherTagId = UUID()
        let port = MockFavoritesPort()
        let (sut, _, _) = makeSUT(port: port)

        await sut.addFavorite(feedItem: makeFeedItem(url: "https://a.com", tagId: tagId), summary: nil, insight: nil)
        await sut.addFavorite(feedItem: makeFeedItem(url: "https://b.com", tagId: otherTagId), summary: nil, insight: nil)
        await sut.addFavorite(feedItem: makeFeedItem(url: "https://c.com"), summary: nil, insight: nil)

        let result = sut.filteredItems(selectedTagId: tagId)

        #expect(result.count == 1)
        #expect(result[0].url == "https://a.com")
    }

    @Test("filteredItems(nil) 재호출: 전체 복원")
    func filteredItemsNil_afterFilter_restoresAll() async {
        let tagId = UUID()
        let port = MockFavoritesPort()
        let (sut, _, _) = makeSUT(port: port)

        await sut.addFavorite(feedItem: makeFeedItem(url: "https://a.com", tagId: tagId), summary: nil, insight: nil)
        await sut.addFavorite(feedItem: makeFeedItem(url: "https://b.com"), summary: nil, insight: nil)

        #expect(sut.filteredItems(selectedTagId: tagId).count == 1)
        #expect(sut.filteredItems(selectedTagId: nil).count == 2)
    }
}
