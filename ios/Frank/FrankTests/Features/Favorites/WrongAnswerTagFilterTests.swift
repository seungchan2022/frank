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

    @Test("favorites에 없는 오답은 어떤 태그 선택에도 항상 표시")
    func apply_wrongAnswerNotInFavorites_alwaysShown() {
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

        // tagId에 맞는 a.com + favorites 미등록 unknown.com 모두 표시
        #expect(result.count == 2)
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
}
