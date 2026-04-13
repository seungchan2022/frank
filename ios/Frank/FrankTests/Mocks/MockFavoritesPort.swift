import Foundation
@testable import Frank

final class MockFavoritesPort: FavoritesPort, @unchecked Sendable {
    // 인메모리 스토어: url → FavoriteItem
    private var store: [String: FavoriteItem] = [:]
    private var insertOrder: [String] = []

    var addCallCount = 0
    var deleteCallCount = 0
    var listCallCount = 0
    var shouldFail = false
    var shouldConflict = false

    /// MVP9 M2: listFavorites 호출 시 반환할 고정 아이템 목록.
    /// nil이면 인메모리 store에서 반환한다.
    var stubItems: [FavoriteItem]? = nil

    func addFavorite(item: FeedItem, summary: String?, insight: String?) async throws -> FavoriteItem {
        addCallCount += 1
        if shouldFail { throw MockFavoritesError.generic }
        if shouldConflict || store[item.url.absoluteString] != nil {
            throw APIFavoritesError.conflict
        }

        let now = Date()
        let fav = FavoriteItem(
            id: UUID(),
            userId: UUID(),
            title: item.title,
            url: item.url.absoluteString,
            snippet: item.snippet,
            source: item.source,
            publishedAt: item.publishedAt,
            tagId: item.tagId,
            summary: summary,
            insight: insight,
            likedAt: now,
            createdAt: now,
            imageUrl: item.imageUrl?.absoluteString,
            quizCompleted: false
        )
        store[fav.url] = fav
        insertOrder.append(fav.url)
        return fav
    }

    func deleteFavorite(url: String) async throws {
        deleteCallCount += 1
        if shouldFail { throw MockFavoritesError.generic }
        store.removeValue(forKey: url)
        insertOrder.removeAll { $0 == url }
    }

    func listFavorites() async throws -> [FavoriteItem] {
        listCallCount += 1
        if shouldFail { throw MockFavoritesError.generic }
        // stubItems가 있으면 우선 반환 (테스트 시나리오용)
        if let stub = stubItems { return stub }
        // 삽입 역순 (created_at DESC 모사)
        return insertOrder.reversed().compactMap { store[$0] }
    }

    var markQuizCompletedCallCount = 0
    var markedQuizUrls: [String] = []

    func markQuizCompleted(url: String) async throws {
        markQuizCompletedCallCount += 1
        if shouldFail { throw MockFavoritesError.generic }
        markedQuizUrls.append(url)
    }
}

enum MockFavoritesError: Error {
    case generic
}
