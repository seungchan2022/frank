import Foundation

protocol TagPort: Sendable {
    func fetchAllTags() async throws -> [Tag]
    func fetchMyTagIds() async throws -> [UUID]
    func saveMyTags(tagIds: [UUID]) async throws
}

extension TagPort {
    /// 전체 태그 + 내 선택 태그 ID를 **병렬**로 가져온다.
    /// OnboardingFeature, SettingsFeature 등 양쪽에서 동일 패턴 중복 → 단일 helper로 추출.
    func fetchAllAndMyTagIds() async throws -> (allTags: [Tag], myTagIds: [UUID]) {
        async let allTags = fetchAllTags()
        async let myIds = fetchMyTagIds()
        return try await (allTags, myIds)
    }
}
