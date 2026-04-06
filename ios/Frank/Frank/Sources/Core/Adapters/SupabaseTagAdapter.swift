import Foundation
import Supabase

struct SupabaseTagAdapter: TagPort {
    private let client: SupabaseClient

    init(client: SupabaseClient) {
        self.client = client
    }

    func fetchAllTags() async throws -> [Tag] {
        let response: [TagDTO] = try await client.from("tags")
            .select()
            .execute()
            .value
        return response.map { $0.toDomain() }
    }

    func fetchMyTagIds() async throws -> [UUID] {
        let userId = try await currentUserId()
        let response: [UserTagDTO] = try await client.from("user_tags")
            .select("tag_id")
            .eq("user_id", value: userId)
            .execute()
            .value
        return response.map(\.tagId)
    }

    func saveMyTags(tagIds: [UUID]) async throws {
        let userId = try await currentUserId()

        try await client.from("user_tags")
            .delete()
            .eq("user_id", value: userId)
            .execute()

        if !tagIds.isEmpty {
            let rows = tagIds.map { UserTagInsert(userId: userId, tagId: $0) }
            try await client.from("user_tags")
                .insert(rows)
                .execute()
        }
    }

    // MARK: - Private

    private func currentUserId() async throws -> UUID {
        let session = try await client.auth.session
        return session.user.id
    }
}

// MARK: - DTOs

private struct TagDTO: Decodable {
    let id: UUID
    let name: String
    let slug: String

    func toDomain() -> Tag {
        Tag(id: id, name: name, slug: slug)
    }
}

private struct UserTagDTO: Decodable {
    let tagId: UUID

    enum CodingKeys: String, CodingKey {
        case tagId = "tag_id"
    }
}

private struct UserTagInsert: Encodable {
    let userId: UUID
    let tagId: UUID

    enum CodingKeys: String, CodingKey {
        case userId = "user_id"
        case tagId = "tag_id"
    }
}
