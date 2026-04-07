import Foundation

struct Article: Identifiable, Equatable, Sendable {
    let id: UUID
    let userId: UUID
    let tagId: UUID?
    let title: String
    let titleKo: String?
    let url: URL
    let snippet: String?
    let source: String
    let searchQuery: String?
    let summary: String?
    let insight: String?
    let summarizedAt: Date?
    let publishedAt: Date?
    let createdAt: Date?

    init(
        id: UUID,
        userId: UUID = UUID(),
        title: String,
        url: URL,
        source: String,
        publishedAt: Date? = nil,
        summary: String? = nil,
        tagId: UUID? = nil,
        titleKo: String? = nil,
        insight: String? = nil,
        snippet: String? = nil,
        summarizedAt: Date? = nil,
        searchQuery: String? = nil,
        createdAt: Date? = nil
    ) {
        self.id = id
        self.userId = userId
        self.title = title
        self.url = url
        self.source = source
        self.publishedAt = publishedAt
        self.summary = summary
        self.tagId = tagId
        self.titleKo = titleKo
        self.insight = insight
        self.snippet = snippet
        self.summarizedAt = summarizedAt
        self.searchQuery = searchQuery
        self.createdAt = createdAt
    }
}
