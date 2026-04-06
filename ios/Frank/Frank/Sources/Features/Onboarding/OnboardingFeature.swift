import Foundation
import Observation

enum OnboardingState: Equatable {
    case loading
    case loaded(tags: [Tag], selectedIds: Set<UUID>)
}

enum OnboardingAction {
    case loadTags
    case toggleTag(id: UUID)
    case complete
}

@Observable
@MainActor
final class OnboardingFeature {
    private(set) var state: OnboardingState = .loading
    private(set) var isSaving = false
    private(set) var errorMessage: String?

    var canComplete: Bool {
        guard case let .loaded(_, selectedIds) = state else { return false }
        return !selectedIds.isEmpty && !isSaving
    }

    private let tag: any TagPort
    private let auth: any AuthPort
    private let onComplete: @MainActor () -> Void

    init(tag: any TagPort, auth: any AuthPort, onComplete: @escaping @MainActor () -> Void) {
        self.tag = tag
        self.auth = auth
        self.onComplete = onComplete
    }

    func send(_ action: OnboardingAction) async {
        switch action {
        case .loadTags:
            await loadTags()
        case let .toggleTag(id):
            toggleTag(id: id)
        case .complete:
            await complete()
        }
    }

    func clearError() {
        errorMessage = nil
    }

    // MARK: - Private

    private func loadTags() async {
        state = .loading
        errorMessage = nil
        do {
            async let allTags = tag.fetchAllTags()
            async let myIds = tag.fetchMyTagIds()
            let (tags, ids) = try await (allTags, myIds)
            state = .loaded(tags: tags, selectedIds: Set(ids))
        } catch {
            state = .loaded(tags: [], selectedIds: Set())
            errorMessage = "태그를 불러오지 못했습니다. 다시 시도해주세요."
        }
    }

    private func toggleTag(id: UUID) {
        guard case let .loaded(tags, selectedIds) = state else { return }
        var updated = selectedIds
        if updated.contains(id) {
            updated.remove(id)
        } else {
            updated.insert(id)
        }
        state = .loaded(tags: tags, selectedIds: updated)
    }

    private func complete() async {
        guard case let .loaded(_, selectedIds) = state else { return }
        guard !selectedIds.isEmpty else { return }

        isSaving = true
        errorMessage = nil
        do {
            try await tag.saveMyTags(tagIds: Array(selectedIds))
            _ = try await auth.updateOnboardingCompleted()
            isSaving = false
            onComplete()
        } catch {
            isSaving = false
            errorMessage = "저장에 실패했습니다. 다시 시도해주세요."
        }
    }
}
