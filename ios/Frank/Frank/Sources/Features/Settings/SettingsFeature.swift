import Foundation
import Observation

enum SettingsAction {
    case loadTags
    case toggleTag(UUID)
    case saveTags
    case signOut
    /// MVP15 M3: 직업 로드
    case loadOccupation
    /// MVP15 M3: 직업 저장. nil = 삭제, 빈 문자열도 nil 처리
    case saveOccupation(String?)
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

    // MARK: - MVP15 M3: Occupation
    /// 현재 서버에 저장된 직업 (로드 후 반영)
    private(set) var occupation: String? = nil
    /// 저장 중 플래그
    private(set) var isSavingOccupation = false
    /// 저장 성공 메시지
    private(set) var occupationSuccess: String? = nil
    /// 저장 오류 메시지
    private(set) var occupationError: String? = nil

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
        case .loadOccupation:
            await loadOccupation()
        case let .saveOccupation(value):
            await saveOccupation(value)
        }
    }

    // MARK: - Private

    private func loadTags() async {
        isLoading = true
        errorMessage = nil
        do {
            let (fetchedTags, fetchedIds) = try await tag.fetchAllAndMyTagIds()
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

    // MARK: - MVP15 M3: Occupation

    private func loadOccupation() async {
        do {
            let profile = try await auth.currentProfile()
            occupation = profile?.occupation
        } catch {
            // 로드 실패는 무시 — occupation nil로 유지
        }
    }

    private func saveOccupation(_ raw: String?) async {
        // 공백·개행 trim 후 빈 문자열은 nil 처리 (삭제 의미)
        let trimmedStr = raw?.trimmingCharacters(in: .whitespacesAndNewlines)
        let normalized: String? = trimmedStr.flatMap { $0.isEmpty ? nil : $0 }
        // 메시지 상태 초기화 — 항상 early return 전에 처리
        occupationError = nil
        occupationSuccess = nil
        // 클라이언트 50자 검증 — 서버 400을 기다리지 않고 즉시 피드백
        if let value = normalized, value.count > 50 {
            occupationError = "직업은 50자 이내로 입력해주세요."
            return
        }
        isSavingOccupation = true
        do {
            let updated = try await auth.updateOccupation(normalized)
            occupation = updated.occupation
            occupationSuccess = "직업이 저장되었습니다."
            isSavingOccupation = false
        } catch {
            isSavingOccupation = false
            occupationError = "직업 저장에 실패했습니다."
        }
    }
}
