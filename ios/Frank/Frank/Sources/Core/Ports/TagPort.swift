import Foundation

protocol TagPort: Sendable {
    func fetchAllTags() async throws -> [Tag]
    func fetchMyTagIds() async throws -> [UUID]
    func saveMyTags(tagIds: [UUID]) async throws
}
