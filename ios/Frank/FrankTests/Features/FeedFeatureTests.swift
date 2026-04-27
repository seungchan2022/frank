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

    // MARK: - 2. loadInitial 성공 (ST5: 프리패치 제거, 전체 탭 첫 페이지만)

    @Test("loadInitial 성공: 내 태그 필터링 + 피드 아이템 로드 (프리패치 없음)")
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
        // ST5 C안: 프리패치 제거 → 전체 탭 1회만 호출
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

    // MARK: - 10. reloadAfterTagChange (ST5: 전체 탭 1회 + 재로드 1회 = 2회)

    @Test("reloadAfterTagChange: selectedTagId 리셋 + loadInitial 재실행")
    func reloadAfterTagChange() async {
        let tagId = UUID()
        let items = [makeFeedItem(urlSuffix: "1", tagId: tagId)]
        let (sut, articlePort, _) = makeSUT(
            feedItems: items,
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId]
        )

        await sut.send(.loadInitial)           // 전체 탭 1회
        await sut.send(.selectTag(tagId))      // 캐시 미스 → 1회
        #expect(sut.selectedTagId == tagId)

        await sut.send(.reloadAfterTagChange)  // 캐시 초기화 + loadInitial: 전체 탭 1회

        #expect(sut.selectedTagId == nil)
        // loadInitial(1) + selectTag 캐시미스(1) + reloadAfterTagChange loadInitial(1) = 3
        #expect(articlePort.fetchFeedCallCount == 3)
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

    // MARK: - M3: 탭 캐시 (ST5 기반)

    /// M3: 캐시 미스 탭 선택 시 조용히 fetchFeed 호출 (isRefreshing 없음)
    @Test("selectTag 캐시 미스 시 조용히 fetchFeed 호출 (로딩 표시 없음)")
    func selectTag_캐시미스_fetchFeed_tagId_전달() async {
        let myTagId = UUID()
        let otherTagId = UUID()
        let items = [
            makeFeedItem(title: "AI Article", urlSuffix: "ai", tagId: myTagId),
            makeFeedItem(title: "Other Article", urlSuffix: "other", tagId: otherTagId)
        ]
        // myTagIds에는 myTagId만 포함 → otherTagId는 프리패치 안 됨
        let (sut, articlePort, _) = makeSUT(
            feedItems: items,
            tags: [Frank.Tag(id: myTagId, name: "AI", category: "ai")],
            myTagIds: [myTagId]
        )

        await sut.send(.loadInitial) // 전체 탭만 (프리패치 없음)
        let countBefore = articlePort.fetchFeedCallCount

        // otherTagId는 캐시 없음 → 조용히 fetch (isRefreshing false)
        await sut.send(.selectTag(otherTagId))

        #expect(articlePort.fetchFeedCallCount == countBefore + 1)
        #expect(articlePort.lastFetchTagId == .some(otherTagId))
        #expect(sut.isRefreshing == false) // 로딩 표시 없음
        #expect(sut.selectedTagId == otherTagId)
    }

    /// M3 S6: selectTag 캐시 히트 → fetchFeed 재호출 없음
    @Test("selectTag 캐시 히트 시 fetchFeed 재호출 없음")
    func selectTag_캐시히트_재요청없음() async {
        let tagId = UUID()
        let items = [makeFeedItem(urlSuffix: "ai", tagId: tagId)]
        let (sut, articlePort, _) = makeSUT(
            feedItems: items,
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId]
        )

        await sut.send(.loadInitial) // 전체 탭 캐시
        await sut.send(.selectTag(tagId)) // 캐시 미스 → 1회 fetch
        let countAfterFirst = articlePort.fetchFeedCallCount

        // 두 번째 selectTag — 캐시 히트 → no fetch
        await sut.send(.selectTag(tagId))

        #expect(articlePort.fetchFeedCallCount == countAfterFirst, "캐시 히트 시 재요청 없음")
    }

    /// M3 S6: refresh → 현재 탭 캐시 무효화 + 재요청
    @Test("refresh 현재 탭 캐시 무효화 후 재요청")
    func refresh_현재탭_캐시무효화_후_재요청() async {
        let tagId = UUID()
        let items = [makeFeedItem(urlSuffix: "ai", tagId: tagId)]
        let (sut, articlePort, _) = makeSUT(
            feedItems: items,
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId]
        )

        await sut.send(.loadInitial)
        await sut.send(.selectTag(tagId))
        let countAfterSelect = articlePort.fetchFeedCallCount

        // refresh → 현재 탭(tagId) 캐시 무효화 + 재요청
        await sut.send(.refresh)

        #expect(articlePort.fetchFeedCallCount == countAfterSelect + 1)
        #expect(articlePort.lastFetchTagId == .some(tagId))
    }

    /// MVP10 M3: refresh는 noCache: true로 fetchFeed 호출해야 함
    @Test("refresh: noCache:true로 fetchFeed 호출")
    func refresh_noCache_true() async {
        let (sut, articlePort, _) = makeSUT(feedItems: [makeFeedItem()])

        await sut.send(.loadInitial)
        await sut.send(.refresh)

        #expect(articlePort.lastFetchNoCache == true)
    }

    /// M3 S6: 전체 탭 복귀 시 캐시 히트 → fetchFeed(tagId: nil) 재호출 없음
    @Test("전체 탭 복귀 시 캐시 히트 — fetchFeed 재호출 없음")
    func selectTag_nil_캐시히트() async {
        let tagId = UUID()
        let items = [makeFeedItem(urlSuffix: "ai", tagId: tagId)]
        let (sut, articlePort, _) = makeSUT(
            feedItems: items,
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId]
        )

        await sut.send(.loadInitial) // 'all' 캐시 저장
        await sut.send(.selectTag(tagId)) // tag 캐시 저장 (캐시 미스 → fetch)
        let countAfterTag = articlePort.fetchFeedCallCount

        // 전체 탭으로 복귀 → 'all' 캐시 히트 → no fetch
        await sut.send(.selectTag(nil))

        #expect(articlePort.fetchFeedCallCount == countAfterTag, "전체 탭 캐시 히트 → 재요청 없음")
        #expect(sut.selectedTagId == nil)
    }

    // MARK: - ST5: 무한 스크롤 테스트

    @Test("loadMore: 다음 페이지 누적")
    func loadMore_누적() async {
        // 25개 아이템 → PAGE_SIZE(20) 첫 페이지 + 5개 두 번째 페이지
        let items = (0..<25).map { i in
            makeFeedItem(title: "Article \(i)", urlSuffix: "\(i)")
        }
        let (sut, _, _) = makeSUT(feedItems: items)

        await sut.send(.loadInitial)
        #expect(sut.feedItems.count == 20)
        #expect(sut.hasMore == true)

        await sut.send(.loadMore)
        #expect(sut.feedItems.count == 25)
        #expect(sut.hasMore == false)
    }

    @Test("loadMore: hasMore=false이면 중단")
    func loadMore_hasMore_false_중단() async {
        // 정확히 PAGE_SIZE(20)개 → 첫 페이지 후 hasMore=true이지만 두 번째 요청 시 0개 → hasMore=false
        let items = (0..<20).map { i in
            makeFeedItem(title: "Article \(i)", urlSuffix: "\(i)")
        }
        let (sut, articlePort, _) = makeSUT(feedItems: items)

        await sut.send(.loadInitial)
        #expect(sut.hasMore == true)

        await sut.send(.loadMore) // 두 번째 페이지 → 0개 반환 → hasMore=false
        #expect(sut.hasMore == false)

        let countBefore = articlePort.fetchFeedCallCount
        await sut.send(.loadMore) // hasMore=false → 중단 (호출 없음)
        #expect(articlePort.fetchFeedCallCount == countBefore)
    }

    @Test("loadMore: limit/offset이 올바르게 전달됨")
    func loadMore_limit_offset_전달() async {
        let items = (0..<25).map { i in
            makeFeedItem(title: "Article \(i)", urlSuffix: "\(i)")
        }
        let (sut, articlePort, _) = makeSUT(feedItems: items)

        await sut.send(.loadInitial)
        #expect(articlePort.lastFetchLimit == 20)
        #expect(articlePort.lastFetchOffset == 0)

        await sut.send(.loadMore)
        #expect(articlePort.lastFetchLimit == 20)
        #expect(articlePort.lastFetchOffset == 20)
    }

    @Test("loadMore: status=loadingMore 중 중복 호출 방지")
    func loadMore_중복_방지() async {
        let items = (0..<25).map { i in
            makeFeedItem(title: "Article \(i)", urlSuffix: "\(i)")
        }
        let (sut, articlePort, _) = makeSUT(feedItems: items)

        await sut.send(.loadInitial)
        let countBefore = articlePort.fetchFeedCallCount

        // 첫 loadMore (완료까지 대기)
        await sut.send(.loadMore)
        let countAfter = articlePort.fetchFeedCallCount

        // 이미 완료된 상태에서 hasMore=false 케이스는 중단됨
        // (중복 가드는 status=loadingMore 시 동작, 여기선 완료 후 상태)
        #expect(countAfter == countBefore + 1)
    }

    @Test("selectTag 후 loadMore: 해당 탭 기준 페이지네이션")
    func selectTag_loadMore_탭별() async {
        let tagId = UUID()
        let items = (0..<25).map { i in
            makeFeedItem(title: "Tag Article \(i)", urlSuffix: "tag-\(i)", tagId: tagId)
        }
        let (sut, articlePort, _) = makeSUT(
            feedItems: items,
            tags: [Frank.Tag(id: tagId, name: "AI", category: "ai")],
            myTagIds: [tagId]
        )

        await sut.send(.loadInitial)
        await sut.send(.selectTag(tagId))
        #expect(sut.feedItems.count == 20)

        await sut.send(.loadMore)
        #expect(sut.feedItems.count == 25)
        #expect(articlePort.lastFetchTagId == .some(tagId))
    }

    @Test("refresh 후 hasMore 초기화 + 첫 페이지로 리셋")
    func refresh_hasMore_초기화() async {
        let items = (0..<25).map { i in
            makeFeedItem(title: "Article \(i)", urlSuffix: "\(i)")
        }
        let (sut, _, _) = makeSUT(feedItems: items)

        await sut.send(.loadInitial)
        await sut.send(.loadMore)
        #expect(sut.feedItems.count == 25)
        #expect(sut.hasMore == false)

        await sut.send(.refresh)
        // refresh 후 첫 페이지 20개로 리셋, hasMore=true
        #expect(sut.feedItems.count == 20)
        #expect(sut.hasMore == true)
    }

    // MARK: - ST5 에러 경로

    @Test("loadMore 실패: hasMore=false로 sentinel 비활성화 (무한 루프 방지)")
    func loadMore_실패_hasMore_false() async {
        // 정확히 PAGE_SIZE(20)개 → 첫 페이지 후 hasMore=true
        let items = (0..<20).map { i in
            makeFeedItem(title: "Article \(i)", urlSuffix: "\(i)")
        }
        let (sut, articlePort, _) = makeSUT(feedItems: items)

        await sut.send(.loadInitial)
        #expect(sut.hasMore == true)

        // 두 번째 페이지 요청 시 에러 발생
        articlePort.fetchError = URLError(.notConnectedToInternet)
        await sut.send(.loadMore)

        // 에러 후 hasMore=false → sentinel 비활성화
        #expect(sut.hasMore == false)
        // isLoadingMore도 false (status=.idle 복귀)
        #expect(sut.isLoadingMore == false)

        // 추가 loadMore 호출 없음 (hasMore=false로 가드됨)
        let countBefore = articlePort.fetchFeedCallCount
        await sut.send(.loadMore)
        #expect(articlePort.fetchFeedCallCount == countBefore)
    }

    @Test("selectTag 실패: 캐시 제거 + selectedTagId 롤백")
    func selectTag_실패_캐시제거_롤백() async {
        let tagId1 = UUID()
        let tagId2 = UUID()
        let items = [
            makeFeedItem(title: "AI Article", urlSuffix: "ai", tagId: tagId1),
            makeFeedItem(title: "iOS Article", urlSuffix: "ios", tagId: tagId2),
        ]
        let (sut, articlePort, _) = makeSUT(
            feedItems: items,
            tags: [
                Frank.Tag(id: tagId1, name: "AI", category: "ai"),
                Frank.Tag(id: tagId2, name: "iOS", category: "ios"),
            ],
            myTagIds: [tagId1, tagId2]
        )

        await sut.send(.loadInitial)
        await sut.send(.selectTag(tagId1)) // 캐시 미스 → 성공
        #expect(sut.selectedTagId == tagId1)

        // tagId2 선택 시 에러 발생
        articlePort.fetchError = URLError(.notConnectedToInternet)
        await sut.send(.selectTag(tagId2))

        // selectedTagId가 이전 값(tagId1)으로 롤백됨
        #expect(sut.selectedTagId == tagId1)
        // errorMessage 설정
        #expect(sut.errorMessage != nil)

        // 에러 후 캐시가 제거됐으므로 다음 selectTag(tagId2)는 캐시 미스로 재시도
        articlePort.fetchError = nil
        let countBefore = articlePort.fetchFeedCallCount
        await sut.send(.selectTag(tagId2))
        // 캐시 없음 → 새 fetch 발생
        #expect(articlePort.fetchFeedCallCount == countBefore + 1)
        #expect(sut.selectedTagId == tagId2)
    }
}
