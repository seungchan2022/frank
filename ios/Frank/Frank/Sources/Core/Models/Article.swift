import Foundation

struct Article: Identifiable, Equatable, Sendable {
    let id: UUID
    let title: String
    let url: URL
    let source: String
    let publishedAt: Date
    let summary: String?
    let tagId: UUID?
    let titleKo: String?
    let insight: String?
    let snippet: String?
    let summarizedAt: Date?

    init(
        id: UUID,
        title: String,
        url: URL,
        source: String,
        publishedAt: Date,
        summary: String? = nil,
        tagId: UUID? = nil,
        titleKo: String? = nil,
        insight: String? = nil,
        snippet: String? = nil,
        summarizedAt: Date? = nil
    ) {
        self.id = id
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
    }
}
