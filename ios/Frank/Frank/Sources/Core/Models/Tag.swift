import Foundation

struct Tag: Identifiable, Equatable, Sendable {
    let id: UUID
    let name: String
    let slug: String
}
