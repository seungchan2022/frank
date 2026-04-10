import Foundation

/// In-memory FavoritesPort 구현 — FRANK_USE_MOCK=1 모드 전용.
struct MockFavoritesAdapter: FavoritesPort {
    func addFavorite(item: FeedItem, summary: String?, insight: String?) async throws -> FavoriteItem {
        let now = Date()
        return FavoriteItem(
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
            createdAt: now
        )
    }

    func deleteFavorite(url: String) async throws {
        // Mock: no-op
    }

    func listFavorites() async throws -> [FavoriteItem] {
        return []
    }
}
