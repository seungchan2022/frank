import Foundation

struct ArticleFilter: Equatable, Sendable {
    var tagId: UUID?
    var limit: Int = 20
    var offset: Int = 0
}
