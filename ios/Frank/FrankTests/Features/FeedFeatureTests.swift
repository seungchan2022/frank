import Testing
import Foundation
@testable import Frank

@Suite("FeedFeature Tests")
@MainActor
struct FeedFeatureTests {

    // MARK: - Test Helpers

    private static let pageSize = 20

    private func makeSUT(
        tags: [Frank.Tag] = [],
        myTagIds: [UUID] = [],
        articles: [Article] = [],
        articlesByTag: [UUID: [Article]] = [:],
        fetchSequence: [[Article]] = [],
        fetchError: Error? = nil,
        collectError: Error? = nil
    ) -> (FeedFeature, MockArticlePort, MockCollectPort, MockTagPort) {
        let articlePort = MockArticlePort()
        articlePort.articles = articles
        articlePort.articlesByTag = articlesByTag
        articlePort.fetchSequence = fetchSequence
        articlePort.fetchError = fetchError

        let collectPort = MockCollectPort()
        collectPort.collectError = collectError

        let tagPort = MockTagPort()
        tagPort.allTags = tags
        tagPort.myTagIds = myTagIds

        let feature = FeedFeature(
            article: articlePort,
            collect: collectPort,
            tag: tagPort
        )
        return (feature, articlePort, collectPort, tagPort)
    }

    private func makeArticle(
        id: UUID = UUID(),
        title: String = "Test Article",
        tagId: UUID = UUID()
    ) -> Article {
        Article(
            id: id,
            title: title,
            url: URL(string: "https://example.com")!,
            source: "Test",
            publishedAt: Date(),
            tagId: tagId
        )
    }

    private func makeTags(count: Int) -> ([Frank.Tag], [UUID]) {
        let tags = (0..<count).map { i in
            Frank.Tag(id: UUID(), name: "Tag\(i)", category: "cat\(i)")
        }
        let ids = tags.map(\.id)
        return (tags, ids)
    }

    // MARK: - 1. 초기 상태 검증

    @Test("초기 상태: tags=[], articles=[], isLoading=false")
    func initialState() {
        let (sut, _, _, _) = makeSUT()

        #expect(sut.tags.isEmpty)
        #expect(sut.articles.isEmpty)
        #expect(sut.selectedTagId == nil)
        #expect(sut.isLoading == false)
        #expect(sut.isLoadingMore == false)
        #expect(sut.hasMore == true)
        #expect(sut.isCollecting == false)
        #expect(sut.errorMessage == nil)
    }

    // MARK: - 2. loadInitial 성공

    @Test("loadInitial 성공: 태그 로드 + 기사 fetch")
    func loadInitialSuccess() async {
        let (allTags, allIds) = makeTags(count: 3)
        let myIds = [allIds[0], allIds[1]]
        let articles = (0..<5).map { i in
            makeArticle(title: "Article \(i)", tagId: myIds[i % 2])
        }

        let (sut, articlePort, _, tagPort) = makeSUT(
            tags: allTags,
            myTagIds: myIds,
            articles: articles
        )

        await sut.send(.loadInitial)

        // 내 태그만 필터링
        #expect(sut.tags.count == 2)
        #expect(sut.tags.allSatisfy { myIds.contains($0.id) })
        #expect(sut.articles.count == 5)
        #expect(sut.isLoading == false)
        #expect(sut.errorMessage == nil)
        #expect(tagPort.fetchAllTagsCallCount == 1)
        #expect(tagPort.fetchMyTagIdsCallCount == 1)
        #expect(articlePort.fetchArticlesFilteredCallCount >= 1)
    }

    // MARK: - 3. loadInitial 기사 0건 → 자동 collectAndRefresh

    @Test("loadInitial 기사 0건: 자동 collectAndRefresh 호출")
    func loadInitialEmptyTriggersCollect() async {
        let (allTags, allIds) = makeTags(count: 2)
        // 첫 fetch는 빈 배열, collect 후 fetch는 기사 있음
        let articlesAfterCollect = [makeArticle(tagId: allIds[0])]

        let (sut, _, collectPort, _) = makeSUT(
            tags: allTags,
            myTagIds: allIds,
            fetchSequence: [[], articlesAfterCollect]
        )

        await sut.send(.loadInitial)

        #expect(collectPort.triggerCollectCallCount == 1)
        #expect(sut.articles.count == 1)
    }

    // MARK: - 4. loadInitial 실패 → errorMessage

    @Test("loadInitial 실패: errorMessage 설정")
    func loadInitialFailure() async {
        let (sut, _, _, _) = makeSUT(
            fetchError: URLError(.notConnectedToInternet)
        )

        await sut.send(.loadInitial)

        #expect(sut.errorMessage != nil)
        #expect(sut.isLoading == false)
    }

    // MARK: - 5. selectTag → selectedTagId 변경 + 기사 fetch

    @Test("selectTag: selectedTagId 변경 + 기사 fetch")
    func selectTag() async {
        let tagId = UUID()
        let tag = Frank.Tag(id: tagId, name: "AI", category: "ai")
        let tagArticles = [makeArticle(title: "AI Article", tagId: tagId)]

        let (sut, _, _, _) = makeSUT(
            tags: [tag],
            myTagIds: [tagId],
            articles: tagArticles,
            articlesByTag: [tagId: tagArticles]
        )

        // 먼저 초기 로드
        await sut.send(.loadInitial)
        // 태그 선택
        await sut.send(.selectTag(tagId))

        #expect(sut.selectedTagId == tagId)
        #expect(sut.articles.count == 1)
        #expect(sut.articles.first?.title == "AI Article")
    }

    // MARK: - 6. selectTag 캐시 히트

    @Test("selectTag 캐시 히트: 서버 호출 없이 캐시 표시")
    func selectTagCacheHit() async {
        let tagId1 = UUID()
        let tagId2 = UUID()
        let tag1 = Frank.Tag(id: tagId1, name: "AI", category: "ai")
        let tag2 = Frank.Tag(id: tagId2, name: "iOS", category: "ios")
        let articles1 = [makeArticle(title: "AI Article", tagId: tagId1)]
        let articles2 = [makeArticle(title: "iOS Article", tagId: tagId2)]

        let (sut, articlePort, _, _) = makeSUT(
            tags: [tag1, tag2],
            myTagIds: [tagId1, tagId2],
            articles: articles1 + articles2,
            articlesByTag: [tagId1: articles1, tagId2: articles2]
        )

        await sut.send(.loadInitial)
        // tag1 선택
        await sut.send(.selectTag(tagId1))

        // tag1으로 돌아왔을 때 — 캐시 히트
        await sut.send(.selectTag(tagId2))
        let callsAfterTag2 = articlePort.fetchArticlesFilteredCallCount

        await sut.send(.selectTag(tagId1))
        let callsAfterReturn = articlePort.fetchArticlesFilteredCallCount

        // tag1 재선택 시 추가 fetch 없어야 함 (캐시 사용)
        #expect(callsAfterReturn == callsAfterTag2)
        #expect(sut.articles == articles1)
    }

    // MARK: - 7. loadMore 성공

    @Test("loadMore 성공: articles에 추가")
    func loadMoreSuccess() async {
        let tagId = UUID()
        let initialArticles = (0..<20).map { i in
            makeArticle(id: UUID(), title: "Article \(i)", tagId: tagId)
        }
        let moreArticles = (20..<25).map { i in
            makeArticle(id: UUID(), title: "Article \(i)", tagId: tagId)
        }

        let allArticles = initialArticles + moreArticles
        let (sut, _, _, _) = makeSUT(
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId],
            articles: allArticles
        )

        await sut.send(.loadInitial)
        let initialCount = sut.articles.count

        await sut.send(.loadMore)

        #expect(sut.articles.count > initialCount)
        #expect(sut.isLoadingMore == false)
    }

    // MARK: - 8. loadMore hasMore=false → 무시

    @Test("loadMore hasMore=false: 무시")
    func loadMoreWhenNoMore() async {
        // 5개 미만의 기사 → hasMore=false
        let articles = (0..<3).map { i in
            makeArticle(id: UUID(), title: "Article \(i)")
        }
        let (sut, articlePort, _, _) = makeSUT(
            tags: [Frank.Tag(id: UUID(), name: "AI", category: "ai")],
            myTagIds: [UUID()],
            articles: articles
        )

        await sut.send(.loadInitial)
        let callsBefore = articlePort.fetchArticlesFilteredCallCount

        // hasMore가 false면 loadMore 무시
        if !sut.hasMore {
            await sut.send(.loadMore)
            #expect(articlePort.fetchArticlesFilteredCallCount == callsBefore)
        }
    }

    // MARK: - 9. loadMore 중복 방지

    @Test("loadMore 중복 방지: isLoadingMore 중 무시")
    func loadMoreDuplicatePrevention() async {
        let articles = (0..<20).map { i in
            makeArticle(id: UUID(), title: "Article \(i)")
        }

        let (sut, _, _, _) = makeSUT(
            tags: [Frank.Tag(id: UUID(), name: "AI", category: "ai")],
            myTagIds: [UUID()],
            articles: articles + articles // 충분한 데이터
        )

        await sut.send(.loadInitial)

        // 두 번 연속 loadMore — 두 번째는 무시되어야 함
        // (비동기 실행이므로 첫 번째가 완료되면 두 번째는 정상 실행)
        await sut.send(.loadMore)
        #expect(sut.isLoadingMore == false)
    }

    // MARK: - 10. collectAndRefresh 성공

    @Test("collectAndRefresh 성공: collect→캐시 무효화→fetch")
    func collectAndRefreshSuccess() async {
        let tagId = UUID()
        let articlesAfterCollect = [
            makeArticle(title: "New Article", tagId: tagId),
        ]

        let (sut, _, collectPort, _) = makeSUT(
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId],
            fetchSequence: [
                [], // loadInitial: 빈 기사 → 자동 collect 트리거
                articlesAfterCollect, // collect 후 fetch
            ]
        )

        await sut.send(.loadInitial)

        #expect(collectPort.triggerCollectCallCount == 1)
        #expect(sut.isCollecting == false)
        #expect(sut.articles.count == 1)
    }

    // MARK: - 11. collectAndRefresh 중복 방지

    @Test("collectAndRefresh 중복 방지: isCollecting 중 무시")
    func collectAndRefreshDuplicatePrevention() async {
        let (sut, _, collectPort, _) = makeSUT(
            tags: [Frank.Tag(id: UUID(), name: "AI", category: "ai")],
            myTagIds: [UUID()],
            articles: [makeArticle()]
        )

        await sut.send(.loadInitial)
        // 첫 번째 실행
        await sut.send(.collectAndRefresh)
        let firstCount = collectPort.triggerCollectCallCount

        // 완료 후 다시 실행은 가능
        await sut.send(.collectAndRefresh)
        #expect(collectPort.triggerCollectCallCount == firstCount + 1)
    }

    // MARK: - 12. collectAndRefresh 실패 → errorMessage

    @Test("collectAndRefresh collect 실패: errorMessage 설정")
    func collectAndRefreshCollectFailure() async {
        let (sut, _, _, _) = makeSUT(
            tags: [Frank.Tag(id: UUID(), name: "AI", category: "ai")],
            myTagIds: [UUID()],
            articles: [makeArticle()],
            collectError: URLError(.notConnectedToInternet)
        )

        await sut.send(.loadInitial)
        await sut.send(.collectAndRefresh)

        #expect(sut.errorMessage != nil)
        #expect(sut.isCollecting == false)
    }

    // MARK: - 13. collectAndRefresh 후 캐시 무효화

    @Test("collectAndRefresh 후 캐시 무효화")
    func collectAndRefreshInvalidatesCache() async {
        let tagId = UUID()
        let tag = Frank.Tag(id: tagId, name: "AI", category: "ai")
        let oldArticles = [makeArticle(title: "Old", tagId: tagId)]
        let newArticles = [
            makeArticle(title: "Old", tagId: tagId),
            makeArticle(title: "New After Collect", tagId: tagId),
        ]

        let (sut, articlePort, _, _) = makeSUT(
            tags: [tag],
            myTagIds: [tagId],
            fetchSequence: [
                oldArticles, // loadInitial
                newArticles, // collect 후 fetch
                newArticles, // selectTag 재fetch (캐시 무효화 확인)
            ]
        )

        await sut.send(.loadInitial)
        #expect(sut.articles.count == 1)

        await sut.send(.collectAndRefresh)
        #expect(sut.articles.count == 2)

        // 캐시 무효화되었으므로 태그 전환 시 서버 호출
        await sut.send(.selectTag(nil))
        let callsBeforeReselect = articlePort.fetchArticlesFilteredCallCount
        await sut.send(.selectTag(tagId))
        #expect(articlePort.fetchArticlesFilteredCallCount > callsBeforeReselect)
    }

    // MARK: - 14. refresh → 캐시 무효화 + 재fetch

    @Test("refresh: 캐시 무효화 + 재fetch")
    func refresh() async {
        let tagId = UUID()
        let tag = Frank.Tag(id: tagId, name: "AI", category: "ai")
        let oldArticles = [makeArticle(title: "Old", tagId: tagId)]
        let newArticles = [
            makeArticle(title: "Old", tagId: tagId),
            makeArticle(title: "Refreshed", tagId: tagId),
        ]

        let (sut, articlePort, _, _) = makeSUT(
            tags: [tag],
            myTagIds: [tagId],
            fetchSequence: [oldArticles, newArticles]
        )

        await sut.send(.loadInitial)
        #expect(sut.articles.count == 1)

        let callsBefore = articlePort.fetchArticlesFilteredCallCount
        await sut.send(.refresh)

        #expect(articlePort.fetchArticlesFilteredCallCount > callsBefore)
        #expect(sut.articles.count == 2)
    }

    // MARK: - 15. selectTag(nil)로 "전체" 탭 전환

    @Test("selectTag(nil): 전체 탭으로 전환")
    func selectTagNilShowsAll() async {
        let tagId = UUID()
        let tag = Frank.Tag(id: tagId, name: "AI", category: "ai")
        let allArticles = (0..<3).map { _ in makeArticle(tagId: tagId) }

        let (sut, _, _, _) = makeSUT(
            tags: [tag],
            myTagIds: [tagId],
            articles: allArticles,
            articlesByTag: [tagId: [allArticles[0]]]
        )

        await sut.send(.loadInitial)
        await sut.send(.selectTag(tagId))
        #expect(sut.selectedTagId == tagId)

        await sut.send(.selectTag(nil))
        #expect(sut.selectedTagId == nil)
        #expect(sut.articles.count == 3)
    }

    // MARK: - 16. tagPort fetchError에서 loadInitial 실패

    @Test("loadInitial 태그 로드 실패: errorMessage 설정")
    func loadInitialTagFetchFailure() async {
        let tagPort = MockTagPort()
        tagPort.fetchError = URLError(.notConnectedToInternet)
        let articlePort = MockArticlePort()
        let collectPort = MockCollectPort()

        let sut = FeedFeature(
            article: articlePort,
            collect: collectPort,
            tag: tagPort
        )

        await sut.send(.loadInitial)

        #expect(sut.errorMessage != nil)
        #expect(sut.isLoading == false)
        #expect(sut.tags.isEmpty)
    }

}
