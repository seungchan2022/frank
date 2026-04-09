import Foundation

struct Article: Identifiable, Equatable, Sendable {
    let id: UUID
    let userId: UUID
    let tagId: UUID?
    let title: String
    let url: URL
    let snippet: String?
    let source: String
    let publishedAt: Date?
    let createdAt: Date?

    init(
        id: UUID,
        userId: UUID = UUID(),
        title: String,
        url: URL,
        source: String,
        publishedAt: Date? = nil,
        tagId: UUID? = nil,
        snippet: String? = nil,
        createdAt: Date? = nil
    ) {
        self.id = id
        self.userId = userId
        self.title = title
        self.url = url
        self.source = source
        self.publishedAt = publishedAt
        self.tagId = tagId
        self.snippet = snippet
        self.createdAt = createdAt
    }
}
