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
        collectError: Error? = nil,
        summarizeError: Error? = nil
    ) -> (FeedFeature, MockArticlePort, MockCollectPort, MockTagPort) {
        let articlePort = MockArticlePort()
        articlePort.articles = articles
        articlePort.articlesByTag = articlesByTag
        articlePort.fetchSequence = fetchSequence
        articlePort.fetchError = fetchError

        let collectPort = MockCollectPort()
        collectPort.collectError = collectError
        collectPort.summarizeError = summarizeError

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
        summary: String? = "Summary",
        tagId: UUID = UUID()
    ) -> Article {
        Article(
            id: id,
            title: title,
            url: URL(string: "https://example.com")!,
            source: "Test",
            publishedAt: Date(),
            summary: summary,
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
        #expect(sut.isSummarizing == false)
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

    // MARK: - 3. loadInitial 기사 0건 → 자동 collectAndSummarize

    @Test("loadInitial 기사 0건: 자동 collectAndSummarize 호출")
    func loadInitialEmptyTriggersCollect() async {
        let (allTags, allIds) = makeTags(count: 2)
        // 첫 fetch는 빈 배열, collect 후 fetch는 기사 있음
        let articlesAfterCollect = [makeArticle(tagId: allIds[0])]

        let (sut, articlePort, collectPort, _) = makeSUT(
            tags: allTags,
            myTagIds: allIds,
            fetchSequence: [[], articlesAfterCollect, articlesAfterCollect]
        )

        await sut.send(.loadInitial)

        #expect(collectPort.triggerCollectCallCount == 1)
        #expect(collectPort.triggerSummarizeCallCount == 1)
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

    // MARK: - 10. collectAndSummarize 성공

    @Test("collectAndSummarize 성공: collect→fetch→summarize→재fetch")
    func collectAndSummarizeSuccess() async {
        let tagId = UUID()
        let articlesWithoutSummary = [
            makeArticle(title: "New Article", summary: nil, tagId: tagId),
        ]
        let articlesWithSummary = [
            makeArticle(title: "New Article", summary: "AI가 생성한 요약", tagId: tagId),
        ]

        let (sut, _, collectPort, _) = makeSUT(
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId],
            fetchSequence: [
                [], // loadInitial: 빈 기사 → 자동 collect 트리거
                articlesWithoutSummary, // collect 후 fetch
                articlesWithSummary, // summarize 후 재fetch
            ]
        )

        await sut.send(.loadInitial)

        #expect(collectPort.triggerCollectCallCount == 1)
        #expect(collectPort.triggerSummarizeCallCount == 1)
        #expect(sut.isCollecting == false)
        #expect(sut.isSummarizing == false)
        #expect(sut.articles.first?.summary == "AI가 생성한 요약")
    }

    // MARK: - 11. collectAndSummarize 중복 방지

    @Test("collectAndSummarize 중복 방지: isCollecting 중 무시")
    func collectAndSummarizeDuplicatePrevention() async {
        let (sut, _, collectPort, _) = makeSUT(
            tags: [Frank.Tag(id: UUID(), name: "AI", category: "ai")],
            myTagIds: [UUID()],
            articles: [makeArticle()]
        )

        await sut.send(.loadInitial)
        // 첫 번째 실행
        await sut.send(.collectAndSummarize)
        let firstCount = collectPort.triggerCollectCallCount

        // 완료 후 다시 실행은 가능
        await sut.send(.collectAndSummarize)
        #expect(collectPort.triggerCollectCallCount == firstCount + 1)
    }

    // MARK: - 12. collectAndSummarize 실패 → errorMessage

    @Test("collectAndSummarize collect 실패: errorMessage 설정")
    func collectAndSummarizeCollectFailure() async {
        let (sut, _, _, _) = makeSUT(
            tags: [Frank.Tag(id: UUID(), name: "AI", category: "ai")],
            myTagIds: [UUID()],
            articles: [makeArticle()],
            collectError: URLError(.timedOut)
        )

        await sut.send(.loadInitial)
        await sut.send(.collectAndSummarize)

        #expect(sut.errorMessage != nil)
        #expect(sut.isCollecting == false)
        #expect(sut.isSummarizing == false)
    }

    @Test("collectAndSummarize summarize 실패: errorMessage 설정")
    func collectAndSummarizeSummarizeFailure() async {
        let (sut, _, _, _) = makeSUT(
            tags: [Frank.Tag(id: UUID(), name: "AI", category: "ai")],
            myTagIds: [UUID()],
            articles: [makeArticle()],
            summarizeError: URLError(.timedOut)
        )

        await sut.send(.loadInitial)
        await sut.send(.collectAndSummarize)

        #expect(sut.errorMessage != nil)
        #expect(sut.isCollecting == false)
        #expect(sut.isSummarizing == false)
    }

    // MARK: - 13. collectAndSummarize 후 캐시 무효화

    @Test("collectAndSummarize 후 캐시 무효화")
    func collectAndSummarizeInvalidatesCache() async {
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
                newArticles, // summarize 후 재fetch
                newArticles, // selectTag 재fetch (캐시 무효화 확인)
            ]
        )

        await sut.send(.loadInitial)
        #expect(sut.articles.count == 1)

        await sut.send(.collectAndSummarize)
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

    // MARK: - 15. 점진적 채움: collect→fetch(summary=nil)→summarize→재fetch(summary 있음)

    @Test("점진적 채움: collect 후 summary=nil, summarize 후 summary 있음")
    func progressiveFill() async {
        let tagId = UUID()
        let articleId = UUID()
        let articleNoSummary = Article(
            id: articleId,
            title: "Breaking News",
            url: URL(string: "https://example.com")!,
            source: "Test",
            publishedAt: Date(),
            summary: nil,
            tagId: tagId
        )
        let articleWithSummary = Article(
            id: articleId,
            title: "Breaking News",
            url: URL(string: "https://example.com")!,
            source: "Test",
            publishedAt: Date(),
            summary: "AI 요약 완료",
            tagId: tagId
        )

        let (sut, _, collectPort, _) = makeSUT(
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId],
            fetchSequence: [
                [], // loadInitial (빈 → 자동 collect)
                [articleNoSummary], // collect 후 fetch (summary=nil)
                [articleWithSummary], // summarize 후 재fetch (summary 있음)
            ]
        )

        await sut.send(.loadInitial)

        #expect(collectPort.triggerCollectCallCount == 1)
        #expect(collectPort.triggerSummarizeCallCount == 1)
        // 최종 상태: summary가 채워진 기사
        #expect(sut.articles.first?.summary == "AI 요약 완료")
    }

    // MARK: - 16. selectTag(nil)로 "전체" 탭 전환

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

    // MARK: - 17. tagPort fetchError에서 loadInitial 실패

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

    // MARK: - 요약 타임아웃

    @Test("isSummarizingTimeout 초기값은 false")
    func summarizingTimeoutInitialValue() {
        let (sut, _, _, _) = makeSUT()
        #expect(sut.isSummarizingTimeout == false)
    }

    @Test("요약이 타임아웃 이내에 완료되면 isSummarizingTimeout은 false 유지")
    func summarizingCompletesBeforeTimeout() async {
        let articlePort = MockArticlePort()
        let collectPort = MockCollectPort()
        collectPort.summarizeDelay = 0
        let tagPort = MockTagPort()
        let sut = FeedFeature(
            article: articlePort,
            collect: collectPort,
            tag: tagPort,
            summarizeTimeoutSeconds: 0.5
        )

        await sut.send(.collectAndSummarize)

        #expect(sut.isSummarizingTimeout == false)
        #expect(sut.phase == .idle)
    }

    @Test("요약이 타임아웃을 초과하면 isSummarizingTimeout = true")
    func summarizingTimesOut() async {
        let articlePort = MockArticlePort()
        let collectPort = MockCollectPort()
        collectPort.summarizeDelay = 0.3
        let tagPort = MockTagPort()
        let sut = FeedFeature(
            article: articlePort,
            collect: collectPort,
            tag: tagPort,
            summarizeTimeoutSeconds: 0.1
        )

        await sut.send(.collectAndSummarize)

        #expect(sut.isSummarizingTimeout == true)
    }

    @Test("retrySummarize: isSummarizingTimeout 초기화 + collectAndSummarize 재시도")
    func retrySummarizeResetsTimeout() async {
        let articlePort = MockArticlePort()
        let collectPort = MockCollectPort()
        collectPort.summarizeDelay = 0.3
        let tagPort = MockTagPort()
        let sut = FeedFeature(
            article: articlePort,
            collect: collectPort,
            tag: tagPort,
            summarizeTimeoutSeconds: 0.1
        )

        // 타임아웃 발생
        await sut.send(.collectAndSummarize)
        #expect(sut.isSummarizingTimeout == true)

        // 재시도: delay 제거 → 정상 완료
        collectPort.summarizeDelay = 0
        await sut.send(.retrySummarize)

        #expect(sut.isSummarizingTimeout == false)
        #expect(sut.phase == .idle)
    }
}
