import Foundation
import Observation

enum SettingsAction {
    case loadTags
    case toggleTag(UUID)
    case saveTags
    case signOut
}

@Observable
@MainActor
final class SettingsFeature: Identifiable {
    let id = UUID()
    private(set) var tags: [Tag] = []
    private(set) var selectedIds: Set<UUID> = []
    private(set) var originalIds: Set<UUID> = []
    private(set) var isLoading = false
    private(set) var isSaving = false
    private(set) var errorMessage: String?
    private(set) var tagsChanged = false

    var canSave: Bool {
        !selectedIds.isEmpty && selectedIds != originalIds && !isSaving
    }

    private let tag: any TagPort
    private let auth: any AuthPort

    init(tag: any TagPort, auth: any AuthPort) {
        self.tag = tag
        self.auth = auth
    }

    func send(_ action: SettingsAction) async {
        switch action {
        case .loadTags:
            await loadTags()
        case let .toggleTag(id):
            toggleTag(id: id)
        case .saveTags:
            await saveTags()
        case .signOut:
            await signOut()
        }
    }

    // MARK: - Private

    private func loadTags() async {
        isLoading = true
        errorMessage = nil
        do {
            async let allTags = tag.fetchAllTags()
            async let myIds = tag.fetchMyTagIds()
            let (fetchedTags, fetchedIds) = try await (allTags, myIds)
            tags = fetchedTags
            selectedIds = Set(fetchedIds)
            originalIds = Set(fetchedIds)
            isLoading = false
        } catch {
            isLoading = false
            errorMessage = "태그를 불러오지 못했습니다."
        }
    }

    private func toggleTag(id: UUID) {
        if selectedIds.contains(id) {
            guard selectedIds.count > 1 else { return }
            selectedIds.remove(id)
        } else {
            selectedIds.insert(id)
        }
    }

    private func saveTags() async {
        guard canSave else { return }

        isSaving = true
        errorMessage = nil
        do {
            try await tag.saveMyTags(tagIds: Array(selectedIds))
            originalIds = selectedIds
            tagsChanged = true
            isSaving = false
        } catch {
            isSaving = false
            errorMessage = "태그 저장에 실패했습니다."
        }
    }

    private func signOut() async {
        do {
            try await auth.signOut()
        } catch {
            errorMessage = "로그아웃에 실패했습니다."
        }
    }
}
