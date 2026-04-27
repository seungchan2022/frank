import Testing
import Foundation
@testable import Frank

@Suite("WrongAnswerTagFilter Tests — MVP11 M4")
struct WrongAnswerTagFilterTests {

    // MARK: - Helpers

    private func makeFavoriteItem(url: String, tagId: UUID?) -> FavoriteItem {
        FavoriteItem(
            id: UUID(),
            userId: UUID(),
            title: "테스트 기사",
            url: url,
            snippet: nil,
            source: "TestSource",
            publishedAt: nil,
            tagId: tagId,
            summary: nil,
            insight: nil,
            likedAt: nil,
            createdAt: nil,
            imageUrl: nil,
            quizCompleted: false
        )
    }

    private func makeWrongAnswer(articleUrl: String) -> WrongAnswer {
        WrongAnswer(
            id: UUID(),
            userId: UUID(),
            articleUrl: articleUrl,
            articleTitle: "테스트 기사",
            question: "질문?",
            options: ["A", "B"],
            correctIndex: 0,
            userIndex: 1,
            explanation: nil,
            createdAt: Date()
        )
    }

    // MARK: - buildTagMap

    @Test("buildTagMap: tagId 있는 FavoriteItem만 매핑")
    func buildTagMap_onlyIncludesItemsWithTagId() {
        let tagId = UUID()
        let items = [
            makeFavoriteItem(url: "https://a.com", tagId: tagId),
            makeFavoriteItem(url: "https://b.com", tagId: nil)
        ]

        let result = WrongAnswerTagFilter.buildTagMap(from: items)

        #expect(result.count == 1)
        #expect(result["https://a.com"] == tagId)
        #expect(result["https://b.com"] == nil)
    }

    @Test("buildTagMap: 빈 배열 → 빈 맵")
    func buildTagMap_emptyItems() {
        let result = WrongAnswerTagFilter.buildTagMap(from: [])
        #expect(result.isEmpty)
    }

    // MARK: - apply: selectedTagId == nil

    @Test("filteredWrongAnswers(nil): 전체 WrongAnswer 반환")
    func apply_nilTagId_returnsAll() {
        let items = [
            makeWrongAnswer(articleUrl: "https://a.com"),
            makeWrongAnswer(articleUrl: "https://b.com")
        ]
        let tagMap: [String: UUID] = [:]

        let result = WrongAnswerTagFilter.apply(items: items, tagMap: tagMap, selectedTagId: nil)

        #expect(result.count == 2)
    }

    // MARK: - apply: selectedTagId != nil

    @Test("filteredWrongAnswers(tagId): url Set 교집합 기반 필터")
    func apply_withTagId_filtersCorrectly() {
        let tagId = UUID()
        let otherTagId = UUID()
        let tagMap: [String: UUID] = [
            "https://a.com": tagId,
            "https://b.com": otherTagId
        ]
        let items = [
            makeWrongAnswer(articleUrl: "https://a.com"),
            makeWrongAnswer(articleUrl: "https://b.com")
        ]

        let result = WrongAnswerTagFilter.apply(items: items, tagMap: tagMap, selectedTagId: tagId)

        #expect(result.count == 1)
        #expect(result[0].articleUrl == "https://a.com")
    }

    @Test("favorites에 없는 오답은 태그 선택 시 제외 (BUG-F 수정)")
    func apply_wrongAnswerNotInFavorites_excluded() {
        let tagId = UUID()
        let tagMap: [String: UUID] = [
            "https://a.com": tagId
        ]
        // https://unknown.com 은 favorites에 없음
        let items = [
            makeWrongAnswer(articleUrl: "https://a.com"),
            makeWrongAnswer(articleUrl: "https://unknown.com")
        ]

        let result = WrongAnswerTagFilter.apply(items: items, tagMap: tagMap, selectedTagId: tagId)

        // tagId에 맞는 a.com만 표시. favorites 미등록 unknown.com은 제외
        #expect(result.count == 1)
        #expect(result[0].articleUrl == "https://a.com")
    }

    @Test("wrongAnswersTags: FavoriteItems에 tagId가 있을 때 해당 Tag만 반환 (buildTagMap 확인)")
    func buildTagMap_multipleItems_correctMapping() {
        let tagId1 = UUID()
        let tagId2 = UUID()
        let items = [
            makeFavoriteItem(url: "https://a.com", tagId: tagId1),
            makeFavoriteItem(url: "https://b.com", tagId: tagId2),
            makeFavoriteItem(url: "https://c.com", tagId: nil)
        ]

        let result = WrongAnswerTagFilter.buildTagMap(from: items)

        #expect(result.count == 2)
        #expect(result["https://a.com"] == tagId1)
        #expect(result["https://b.com"] == tagId2)
        #expect(result["https://c.com"] == nil)
    }

    // MARK: - wrongAnswerTags join 로직 검증 (BUG-F 핵심)

    /// wrongAnswerTags computed 동등 시나리오:
    /// favorites 3개 (A·B·C 태그), 오답 1개(A 태그 URL) →
    /// wrongAnswerTags = feature.tags.filter { usedTagIds } == [A] (B·C 제외)
    @Test("wrongAnswerTags join: 오답 기사 URL에 매핑된 태그만 반환 (BUG-F)")
    func wrongAnswerTags_join_returnsOnlyTagsMatchingWrongAnswerUrls() {
        let tagA = UUID()
        let tagB = UUID()
        let tagC = UUID()

        // favorites: A·B·C 태그 기사 각 1개
        let favoriteItems = [
            makeFavoriteItem(url: "https://a.com", tagId: tagA),
            makeFavoriteItem(url: "https://b.com", tagId: tagB),
            makeFavoriteItem(url: "https://c.com", tagId: tagC)
        ]

        // 오답: A 태그 기사만 포함
        let wrongAnswers = [
            makeWrongAnswer(articleUrl: "https://a.com")
        ]

        // FavoritesView.wrongAnswerTags 로직 재현
        let tagMap = WrongAnswerTagFilter.buildTagMap(from: favoriteItems)
        let usedTagIds = Set(wrongAnswers.compactMap { tagMap[$0.articleUrl] })

        // feature.tags = [tagA, tagB, tagC] 기준 필터
        let allTagIds = [tagA, tagB, tagC]
        let resultTagIds = allTagIds.filter { usedTagIds.contains($0) }

        // 오답이 A 태그 기사만 참조하므로 tagA만 반환
        #expect(resultTagIds.count == 1)
        #expect(resultTagIds[0] == tagA)
    }
}
