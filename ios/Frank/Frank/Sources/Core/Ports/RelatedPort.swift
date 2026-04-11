import Foundation

/// MVP7 M3: 연관 기사 조회 포트.
/// GET /api/me/articles/related?title=...&snippet=...
protocol RelatedPort: Sendable {
    /// 연관 기사 목록 조회.
    /// - Parameters:
    ///   - title: 현재 기사 제목
    ///   - snippet: 현재 기사 요약 (없으면 nil)
    /// - Returns: 연관 기사 FeedItem 배열
    func fetchRelated(title: String, snippet: String?) async throws -> [FeedItem]
}
