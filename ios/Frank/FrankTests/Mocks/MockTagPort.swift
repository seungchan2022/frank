import Foundation
@testable import Frank

final class MockTagPort: TagPort, @unchecked Sendable {
    var allTags: [Tag] = []
    var myTagIds: [UUID] = []
    var saveError: Error?
    var fetchError: Error?

    var fetchAllTagsCallCount = 0
    var fetchMyTagIdsCallCount = 0
    var saveMyTagsCallCount = 0
    var savedTagIds: [UUID]?

    func fetchAllTags() async throws -> [Tag] {
        fetchAllTagsCallCount += 1
        if let error = fetchError { throw error }
        return allTags
    }

    func fetchMyTagIds() async throws -> [UUID] {
        fetchMyTagIdsCallCount += 1
        return myTagIds
    }

    func saveMyTags(tagIds: [UUID]) async throws {
        saveMyTagsCallCount += 1
        savedTagIds = tagIds
        if let error = saveError { throw error }
    }
}
