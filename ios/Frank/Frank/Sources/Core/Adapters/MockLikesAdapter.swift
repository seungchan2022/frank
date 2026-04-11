import Foundation

/// In-memory LikesPort 구현 — FRANK_USE_MOCK=1 모드 전용.
struct MockLikesAdapter: LikesPort {
    func likeArticle(title: String, snippet: String?) async throws -> LikeResult {
        // Mock: 고정 키워드 반환
        return LikeResult(keywords: ["iOS", "Swift", "SwiftUI"], totalLikes: 1)
    }
}
