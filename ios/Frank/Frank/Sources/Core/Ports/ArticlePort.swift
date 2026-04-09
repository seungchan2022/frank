import Foundation

/// MVP5 M1: ArticlePort → FeedPort 역할.
/// GET /me/feed — 검색 API 직접 호출 결과 반환 (DB 저장 없음).
/// `fetchArticle(id:)` 제거 — M2에서 url 기반 요약 엔드포인트로 대체 예정.
protocol ArticlePort: Sendable {
    func fetchFeed() async throws -> [FeedItem]
}
