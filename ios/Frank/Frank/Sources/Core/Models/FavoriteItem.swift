import Foundation

/// MVP5 M3: 즐겨찾기 아이템 모델.
/// 서버 favorites 테이블과 1:1 대응.
/// UNIQUE (user_id, url)
struct FavoriteItem: Codable, Identifiable, Equatable, Hashable, Sendable {
    let id: UUID
    let userId: UUID
    let title: String
    let url: String
    let snippet: String?
    let source: String
    let publishedAt: Date?
    let tagId: UUID?
    let summary: String?
    let insight: String?
    let likedAt: Date?
    let createdAt: Date?
    /// MVP6 M1: 썸네일 이미지 URL 문자열 (없으면 nil)
    let imageUrl: String?

    enum CodingKeys: String, CodingKey {
        case id
        case userId = "user_id"
        case title
        case url
        case snippet
        case source
        case publishedAt = "published_at"
        case tagId = "tag_id"
        case summary
        case insight
        case likedAt = "liked_at"
        case createdAt = "created_at"
        case imageUrl = "image_url"
    }
}
