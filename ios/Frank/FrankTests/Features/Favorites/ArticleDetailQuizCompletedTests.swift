import Testing
import Foundation
@testable import Frank

/// MVP9 M2: ArticleDetailView quizCompleted 분기 로직 테스트.
///
/// ArticleDetailView는 SwiftUI View라 직접 단위테스트 대신
/// FavoritesFeature에서 quizCompleted 여부 조회 로직을 검증한다.
@Suite("FavoritesFeature quizCompleted 조회 Tests — MVP9 M2")
@MainActor
struct ArticleDetailQuizCompletedTests {

    private func makeFavoriteItem(url: String, quizCompleted: Bool) -> FavoriteItem {
        FavoriteItem(
            id: UUID(),
            userId: UUID(),
            title: "테스트 기사",
            url: url,
            snippet: nil,
            source: "테스트",
            publishedAt: nil,
            tagId: nil,
            summary: nil,
            insight: nil,
            likedAt: nil,
            createdAt: nil,
            imageUrl: nil,
            quizCompleted: quizCompleted
        )
    }

    // MARK: - FavoritesFeature quizCompleted 조회

    @Test("quizCompleted=true인 즐겨찾기 항목 조회")
    func quizCompleted_true_for_completed_item() async {
        let port = MockFavoritesPort()
        let feature = FavoritesFeature(favorites: port, tag: MockTagPort())

        // 수동으로 items 설정 테스트를 위해 Port를 통해 로드
        let url = "https://example.com/article"
        let item = makeFavoriteItem(url: url, quizCompleted: true)
        port.stubItems = [item]

        await feature.loadFavorites()

        let found = feature.items.first { $0.url == url }
        #expect(found?.quizCompleted == true)
    }

    @Test("quizCompleted=false인 즐겨찾기 항목 조회")
    func quizCompleted_false_for_uncompleted_item() async {
        let port = MockFavoritesPort()
        let feature = FavoritesFeature(favorites: port, tag: MockTagPort())

        let url = "https://example.com/article2"
        let item = makeFavoriteItem(url: url, quizCompleted: false)
        port.stubItems = [item]

        await feature.loadFavorites()

        let found = feature.items.first { $0.url == url }
        #expect(found?.quizCompleted == false)
    }

    @Test("URL로 FavoriteItem의 quizCompleted 값을 조회하는 헬퍼 메서드")
    func quizCompleted_helper_returns_correct_value() async {
        let port = MockFavoritesPort()
        let feature = FavoritesFeature(favorites: port, tag: MockTagPort())

        let url = "https://example.com/quiz-done"
        let item = makeFavoriteItem(url: url, quizCompleted: true)
        port.stubItems = [item]

        await feature.loadFavorites()

        #expect(feature.isQuizCompleted(url) == true)
        #expect(feature.isQuizCompleted("https://other.com") == false)
    }

    @Test("즐겨찾기에 없는 기사는 quizCompleted=false 반환")
    func quizCompleted_returns_false_for_unknown_url() {
        let port = MockFavoritesPort()
        let feature = FavoritesFeature(favorites: port, tag: MockTagPort())

        #expect(feature.isQuizCompleted("https://unknown.com") == false)
    }
}
