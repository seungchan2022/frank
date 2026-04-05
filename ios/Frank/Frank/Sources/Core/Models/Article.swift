import Foundation

struct Article: Identifiable, Equatable, Sendable {
    let id: UUID
    let title: String
    let url: URL
    let source: String
    let publishedAt: Date
    let summary: String?
    let tagId: UUID
}
