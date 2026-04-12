import Foundation

/// 즐겨찾기 CRUD 포트.
/// POST/DELETE/GET /api/me/favorites
protocol FavoritesPort: Sendable {
    func addFavorite(item: FeedItem, summary: String?, insight: String?) async throws -> FavoriteItem
    func deleteFavorite(url: String) async throws
    func listFavorites() async throws -> [FavoriteItem]
    /// MVP8 M3: 퀴즈 완료 마킹 — POST /api/me/favorites/quiz/done
    func markQuizCompleted(url: String) async throws
}
