import Testing
import Foundation
@testable import Frank

/// MVP13 M2: favorites 브릿지(tagMap) 완전 제거 → WrongAnswer.tagId 직접 사용 테스트.
/// MVP11 M4 buildTagMap/apply 테스트 제거, filter(items:selectedTagId:) 테스트로 교체.
@Suite("WrongAnswerTagFilter Tests — MVP13 M2")
struct WrongAnswerTagFilterTests {

    // MARK: - Helpers

    private func makeWrongAnswer(articleUrl: String, tagId: UUID?) -> WrongAnswer {
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
            createdAt: Date(),
            tagId: tagId
        )
    }

    // MARK: - filter: selectedTagId == nil

    @Test("filter(nil): 전체 WrongAnswer 반환")
    func filter_nilTagId_returnsAll() {
        let tagId = UUID()
        let items = [
            makeWrongAnswer(articleUrl: "https://a.com", tagId: tagId),
            makeWrongAnswer(articleUrl: "https://b.com", tagId: nil)
        ]

        let result = WrongAnswerTagFilter.filter(items: items, selectedTagId: nil)

        #expect(result.count == 2)
    }

    @Test("filter(nil): 빈 배열 → 빈 배열")
    func filter_nilTagId_emptyItems() {
        let result = WrongAnswerTagFilter.filter(items: [], selectedTagId: nil)
        #expect(result.isEmpty)
    }

    // MARK: - filter: selectedTagId != nil

    @Test("filter(tagId): wa.tagId == selectedTagId 인 항목만 반환")
    func filter_withTagId_filtersCorrectly() {
        let tagId = UUID()
        let otherTagId = UUID()
        let items = [
            makeWrongAnswer(articleUrl: "https://a.com", tagId: tagId),
            makeWrongAnswer(articleUrl: "https://b.com", tagId: otherTagId)
        ]

        let result = WrongAnswerTagFilter.filter(items: items, selectedTagId: tagId)

        #expect(result.count == 1)
        #expect(result[0].articleUrl == "https://a.com")
    }

    @Test("filter(tagId): tagId=nil 오답은 태그 선택 시 제외")
    func filter_wrongAnswerWithNilTagId_excluded() {
        let tagId = UUID()
        let items = [
            makeWrongAnswer(articleUrl: "https://a.com", tagId: tagId),
            makeWrongAnswer(articleUrl: "https://b.com", tagId: nil) // tagId 없음
        ]

        let result = WrongAnswerTagFilter.filter(items: items, selectedTagId: tagId)

        #expect(result.count == 1)
        #expect(result[0].articleUrl == "https://a.com")
    }

    @Test("filter(tagId): 선택된 태그 일치 항목 없으면 빈 배열")
    func filter_noMatchingTagId_returnsEmpty() {
        let tagId = UUID()
        let otherTagId = UUID()
        let items = [
            makeWrongAnswer(articleUrl: "https://a.com", tagId: otherTagId)
        ]

        let result = WrongAnswerTagFilter.filter(items: items, selectedTagId: tagId)

        #expect(result.isEmpty)
    }

    @Test("filter(tagId): 여러 항목 중 일치하는 것만 필터링")
    func filter_multipleItems_correctFiltering() {
        let tagA = UUID()
        let tagB = UUID()
        let tagC = UUID()
        let items = [
            makeWrongAnswer(articleUrl: "https://a.com", tagId: tagA),
            makeWrongAnswer(articleUrl: "https://b.com", tagId: tagB),
            makeWrongAnswer(articleUrl: "https://c.com", tagId: tagA), // tagA 중복
            makeWrongAnswer(articleUrl: "https://d.com", tagId: tagC),
            makeWrongAnswer(articleUrl: "https://e.com", tagId: nil)
        ]

        let result = WrongAnswerTagFilter.filter(items: items, selectedTagId: tagA)

        #expect(result.count == 2)
        #expect(result.allSatisfy { $0.tagId == tagA })
    }

    // MARK: - 회귀: favorites 브릿지 없음 확인

    @Test("favorites 브릿지(tagMap) 없이 직접 tagId 비교")
    func filter_directTagIdComparison_noBridge() {
        let tagId = UUID()
        // 동일 articleUrl이더라도 tagId 기준 필터 (url 기반 favorites 조인 불필요)
        let items = [
            makeWrongAnswer(articleUrl: "https://a.com", tagId: tagId),
            makeWrongAnswer(articleUrl: "https://a.com", tagId: nil) // 같은 URL, tagId 없음
        ]

        let result = WrongAnswerTagFilter.filter(items: items, selectedTagId: tagId)

        // tagId 있는 항목만 반환
        #expect(result.count == 1)
        #expect(result[0].tagId == tagId)
    }
}
