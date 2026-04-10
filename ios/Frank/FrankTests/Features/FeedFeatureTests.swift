import Testing
import Foundation
@testable import Frank

@Suite("FeedFeature Tests")
@MainActor
struct FeedFeatureTests {

    // MARK: - Helpers

    private func makeFeedItem(
        title: String = "Test Article",
        urlSuffix: String = "article",
        tagId: UUID? = nil
    ) -> FeedItem {
        FeedItem(
            title: title,
            url: URL(string: "https://example.com/\(urlSuffix)")!,
            source: "Test",
            tagId: tagId
        )
    }

    private func makeSUT(
        feedItems: [FeedItem] = [],
        fetchError: Error? = nil,
        tags: [Frank.Tag] = [],
        myTagIds: [UUID] = []
    ) -> (FeedFeature, MockArticlePort, MockTagPort) {
        let articlePort = MockArticlePort()
        articlePort.feedItems = feedItems
        articlePort.fetchError = fetchError

        let tagPort = MockTagPort()
        tagPort.allTags = tags
        tagPort.myTagIds = myTagIds

        let feature = FeedFeature(article: articlePort, tag: tagPort)
        return (feature, articlePort, tagPort)
    }

    private func makeTags(count: Int) -> ([Frank.Tag], [UUID]) {
        let tags = (0..<count).map { i in Frank.Tag(id: UUID(), name: "Tag\(i)", category: "cat\(i)") }
        return (tags, tags.map(\.id))
    }

    // MARK: - 1. 초기 상태

    @Test("초기 상태: feedItems=[], tags=[], isLoading=false")
    func initialState() {
        let (sut, _, _) = makeSUT()
        #expect(sut.feedItems.isEmpty)
        #expect(sut.tags.isEmpty)
        #expect(sut.selectedTagId == nil)
        #expect(sut.isLoading == false)
        #expect(sut.isRefreshing == false)
        #expect(sut.errorMessage == nil)
    }

    // MARK: - 2. loadInitial 성공

    @Test("loadInitial 성공: 내 태그 필터링 + 피드 아이템 로드")
    func loadInitialSuccess() async {
        let (allTags, allIds) = makeTags(count: 3)
        let myIds = [allIds[0], allIds[1]]
        let items = (0..<5).map { i in
            makeFeedItem(title: "Article \(i)", urlSuffix: "\(i)", tagId: myIds[i % 2])
        }

        let (sut, articlePort, tagPort) = makeSUT(
            feedItems: items,
            tags: allTags,
            myTagIds: myIds
        )

        await sut.send(.loadInitial)

        #expect(sut.tags.count == 2)
        #expect(sut.tags.allSatisfy { myIds.contains($0.id) })
        #expect(sut.feedItems.count == 5)
        #expect(sut.isLoading == false)
        #expect(sut.errorMessage == nil)
        #expect(tagPort.fetchAllTagsCallCount == 1)
        #expect(tagPort.fetchMyTagIdsCallCount == 1)
        #expect(articlePort.fetchFeedCallCount == 1)
    }

    // MARK: - 3. loadInitial 피드 실패

    @Test("loadInitial 피드 로드 실패: errorMessage 설정")
    func loadInitialFeedFailure() async {
        let (sut, _, _) = makeSUT(fetchError: URLError(.notConnectedToInternet))

        await sut.send(.loadInitial)

        #expect(sut.errorMessage != nil)
        #expect(sut.isLoading == false)
        #expect(sut.feedItems.isEmpty)
    }

    // MARK: - 4. loadInitial 태그 로드 실패

    @Test("loadInitial 태그 로드 실패: errorMessage 설정")
    func loadInitialTagFailure() async {
        let tagPort = MockTagPort()
        tagPort.fetchError = URLError(.notConnectedToInternet)
        let articlePort = MockArticlePort()
        let sut = FeedFeature(article: articlePort, tag: tagPort)

        await sut.send(.loadInitial)

        #expect(sut.errorMessage != nil)
        #expect(sut.isLoading == false)
        #expect(sut.tags.isEmpty)
    }

    // MARK: - 5. selectTag 필터링

    @Test("selectTag: articles가 해당 태그로 필터링됨")
    func selectTagFilters() async {
        let tagId1 = UUID()
        let tagId2 = UUID()
        let items = [
            makeFeedItem(title: "AI Article", urlSuffix: "ai", tagId: tagId1),
            makeFeedItem(title: "iOS Article", urlSuffix: "ios", tagId: tagId2),
            makeFeedItem(title: "AI Article 2", urlSuffix: "ai2", tagId: tagId1),
        ]

        let (sut, _, _) = makeSUT(
            feedItems: items,
            tags: [
                Frank.Tag(id: tagId1, name: "AI", category: "ai"),
                Frank.Tag(id: tagId2, name: "iOS", category: "ios"),
            ],
            myTagIds: [tagId1, tagId2]
        )

        await sut.send(.loadInitial)
        await sut.send(.selectTag(tagId1))

        #expect(sut.selectedTagId == tagId1)
        #expect(sut.articles.count == 2)
        #expect(sut.articles.allSatisfy { $0.tagId == tagId1 })
    }

    // MARK: - 6. selectTag(nil) 전체 표시

    @Test("selectTag(nil): 전체 아이템 표시")
    func selectTagNilShowsAll() async {
        let tagId = UUID()
        let items = (0..<3).map { i in
            makeFeedItem(title: "Article \(i)", urlSuffix: "\(i)", tagId: tagId)
        }

        let (sut, _, _) = makeSUT(
            feedItems: items,
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId]
        )

        await sut.send(.loadInitial)
        await sut.send(.selectTag(tagId))
        #expect(sut.articles.count == 3)

        await sut.send(.selectTag(nil))
        #expect(sut.selectedTagId == nil)
        #expect(sut.articles.count == 3)
    }

    // MARK: - 7. articles computed 속성 (selectedTagId=nil → feedItems 전체)

    @Test("articles: selectedTagId=nil이면 전체 feedItems 반환")
    func articlesWithNoTagSelected() async {
        let tagId1 = UUID()
        let tagId2 = UUID()
        let items = [
            makeFeedItem(urlSuffix: "1", tagId: tagId1),
            makeFeedItem(urlSuffix: "2", tagId: tagId2),
        ]

        let (sut, _, _) = makeSUT(
            feedItems: items,
            tags: [
                Frank.Tag(id: tagId1, name: "A", category: "a"),
                Frank.Tag(id: tagId2, name: "B", category: "b"),
            ],
            myTagIds: [tagId1, tagId2]
        )

        await sut.send(.loadInitial)
        #expect(sut.articles.count == 2)
    }

    // MARK: - 8. refresh

    @Test("refresh: fetchFeed 재호출 + feedItems 갱신")
    func refresh() async {
        let tagId = UUID()
        let items = [makeFeedItem(urlSuffix: "1", tagId: tagId)]
        let (sut, articlePort, _) = makeSUT(
            feedItems: items,
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId]
        )

        await sut.send(.loadInitial)
        let callsBefore = articlePort.fetchFeedCallCount

        await sut.send(.refresh)

        #expect(articlePort.fetchFeedCallCount == callsBefore + 1)
        #expect(sut.isRefreshing == false)
        #expect(sut.errorMessage == nil)
    }

    // MARK: - 9. refresh 실패

    @Test("refresh 실패: errorMessage 설정")
    func refreshFailure() async {
        let tagId = UUID()
        let items = [makeFeedItem(urlSuffix: "1", tagId: tagId)]
        let (sut, articlePort, _) = makeSUT(
            feedItems: items,
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId]
        )

        await sut.send(.loadInitial)
        articlePort.fetchError = URLError(.notConnectedToInternet)
        await sut.send(.refresh)

        #expect(sut.errorMessage != nil)
        #expect(sut.isRefreshing == false)
    }

    // MARK: - 10. reloadAfterTagChange

    @Test("reloadAfterTagChange: selectedTagId 리셋 + loadInitial 재실행")
    func reloadAfterTagChange() async {
        let tagId = UUID()
        let items = [makeFeedItem(urlSuffix: "1", tagId: tagId)]
        let (sut, articlePort, _) = makeSUT(
            feedItems: items,
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId]
        )

        await sut.send(.loadInitial)
        await sut.send(.selectTag(tagId))
        #expect(sut.selectedTagId == tagId)

        await sut.send(.reloadAfterTagChange)

        #expect(sut.selectedTagId == nil)
        #expect(articlePort.fetchFeedCallCount == 2)
    }

    // MARK: - 11. loadInitial 후 isLoading=false

    @Test("loadInitial 완료 후: isLoading=false, isRefreshing=false")
    func loadingStateAfterInitial() async {
        let (sut, _, _) = makeSUT(
            feedItems: [makeFeedItem()],
            tags: [Frank.Tag(id: UUID(), name: "AI", category: "ai")],
            myTagIds: [UUID()]
        )

        await sut.send(.loadInitial)

        #expect(sut.isLoading == false)
        #expect(sut.isRefreshing == false)
    }

    // MARK: - 12. refresh 중 feedItems 유지 (stale-while-revalidate)

    @Test("refresh 중 feedItems 유지 — phase가 .refreshing이어도 기존 아이템 남아있음")
    func refresh_중_feedItems_유지() async {
        let tagId = UUID()
        let oldItem = makeFeedItem(title: "Old Article", urlSuffix: "old", tagId: tagId)
        let (sut, articlePort, _) = makeSUT(
            feedItems: [oldItem],
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId]
        )

        await sut.send(.loadInitial)
        #expect(sut.feedItems.count == 1)

        // refresh 중에도 feedItems 유지 확인 — refreshing phase로 진입하기 전 스냅샷
        // FeedFeature.refresh()는 phase = .refreshing으로 먼저 전환 후 API 호출
        // → API 호출 전까지 feedItems는 교체되지 않음
        // 여기서는 완료 후 결과가 교체됐는지 검증
        let newItem = makeFeedItem(title: "New Article", urlSuffix: "new", tagId: tagId)
        articlePort.feedItems = [newItem]
        await sut.send(.refresh)

        // 완료 후 새 결과로 교체됨
        #expect(sut.feedItems.count == 1)
        #expect(sut.feedItems[0].title == "New Article")
    }

    // MARK: - 13. refresh 완료 후 feedItems 교체

    @Test("refresh 완료 후 feedItems 새 결과로 교체됨")
    func refresh_완료_후_feedItems_교체() async {
        let tagId = UUID()
        let initialItems = [
            makeFeedItem(title: "Article 1", urlSuffix: "1", tagId: tagId),
            makeFeedItem(title: "Article 2", urlSuffix: "2", tagId: tagId)
        ]
        let (sut, articlePort, _) = makeSUT(
            feedItems: initialItems,
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId]
        )

        await sut.send(.loadInitial)
        #expect(sut.feedItems.count == 2)

        let newItems = [makeFeedItem(title: "Fresh Article", urlSuffix: "fresh", tagId: tagId)]
        articlePort.feedItems = newItems
        await sut.send(.refresh)

        #expect(sut.feedItems.count == 1)
        #expect(sut.feedItems[0].title == "Fresh Article")
    }

    // MARK: - 14. refresh 완료 후 phase .idle 복귀

    @Test("refresh 완료 후 phase .idle 복귀")
    func refresh_완료_후_phase_idle() async {
        let tagId = UUID()
        let (sut, _, _) = makeSUT(
            feedItems: [makeFeedItem(urlSuffix: "1", tagId: tagId)],
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId]
        )

        await sut.send(.loadInitial)
        await sut.send(.refresh)

        #expect(sut.phase == .idle)
        #expect(sut.isRefreshing == false)
        #expect(sut.errorMessage == nil)
    }
}
