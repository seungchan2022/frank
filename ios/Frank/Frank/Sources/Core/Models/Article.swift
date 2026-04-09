import Foundation

/// MVP5 M1: FeedItem — ephemeral, DB에 저장되지 않음.
/// GET /me/feed 응답. DB id 없음 — url을 기사 식별자로 사용.
/// `id` 프로퍼티는 SwiftUI List 식별용으로 url absoluteString에서 파생된다.
struct FeedItem: Identifiable, Equatable, Sendable {
    /// SwiftUI Identifiable 준수용 — url absoluteString 기반
    var id: String { url.absoluteString }

    let title: String
    let url: URL
    let snippet: String?
    let source: String
    let publishedAt: Date?
    let tagId: UUID?

    init(
        title: String,
        url: URL,
        source: String,
        publishedAt: Date? = nil,
        tagId: UUID? = nil,
        snippet: String? = nil
    ) {
        self.title = title
        self.url = url
        self.source = source
        self.publishedAt = publishedAt
        self.tagId = tagId
        self.snippet = snippet
    }
}

/// MVP5 M1: 하위 호환 타입 별칭.
/// FeedView, ArticleCardView, ArticleDetailView 등 Article 참조를 유지하면서
/// M2에서 완전히 FeedItem으로 전환 예정.
typealias Article = FeedItem
