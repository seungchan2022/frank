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

    /// MVP8 M3: 퀴즈 완료 여부 (한 번이라도 퀴즈를 풀었으면 true)
    let quizCompleted: Bool
    /// MVP15 M3: 직업 시각으로 재작성된 텍스트 (없으면 nil)
    let rewrite: String?
    /// MVP16 M3: 재작성 당시의 occupation. nil = 미추적 또는 재작성 없음.
    /// 현재 occupation과 비교해 재작성 버튼 재활성화 여부를 판단.
    let rewriteOccupation: String?

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
        case quizCompleted = "quiz_completed"
        case rewrite
        case rewriteOccupation = "rewrite_occupation"
    }
}
