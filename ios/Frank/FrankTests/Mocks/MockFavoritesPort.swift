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
            imageUrl: item.imageUrl?.absoluteString
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
        // 삽입 역순 (created_at DESC 모사)
        return insertOrder.reversed().compactMap { store[$0] }
    }
}

enum MockFavoritesError: Error {
    case generic
}
