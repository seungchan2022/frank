import Foundation

/// In-memory TagPort 구현. fixture 기반.
actor MockTagAdapter: TagPort {
    private var allTags: [Tag]
    private var myTagIds: [UUID]

    init(
        seed: [Tag] = MockFixtures.tags,
        initialMyTags: [UUID] = [MockFixtures.tagAIML, MockFixtures.tagIOS]
    ) {
        self.allTags = seed
        self.myTagIds = initialMyTags
    }

    func fetchAllTags() async throws -> [Tag] {
        allTags
    }

    func fetchMyTagIds() async throws -> [UUID] {
        myTagIds
    }

    func saveMyTags(tagIds: [UUID]) async throws {
        myTagIds = tagIds
    }
}
