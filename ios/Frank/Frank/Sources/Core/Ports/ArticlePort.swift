import Foundation

/// MVP5 M1: ArticlePort → FeedPort 역할.
/// GET /me/feed — 검색 API 직접 호출 결과 반환 (DB 저장 없음).
/// `fetchArticle(id:)` 제거 — M2에서 url 기반 요약 엔드포인트로 대체 예정.
protocol ArticlePort: Sendable {
    /// MVP6 M3: tagId 있으면 해당 태그만 서버에서 필터링. nil = 전체
    /// MVP10 M3: noCache=true → Cache-Control: no-cache 헤더 전송 (pull-to-refresh 캐시 우회)
    func fetchFeed(tagId: UUID?, noCache: Bool) async throws -> [FeedItem]
}
