import Foundation

/// MVP7 M2: 좋아요 처리 포트.
/// POST /api/me/articles/like
protocol LikesPort: Sendable {
    /// 기사 좋아요 처리.
    /// - Returns: 추출된 키워드 + 누적 좋아요 수
    func likeArticle(title: String, snippet: String?) async throws -> LikeResult
}

/// 좋아요 처리 결과
struct LikeResult: Equatable {
    let keywords: [String]
    let totalLikes: Int
}
