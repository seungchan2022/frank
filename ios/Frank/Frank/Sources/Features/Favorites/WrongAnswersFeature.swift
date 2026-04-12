import Foundation
import Observation

/// MVP8 M3: WrongAnswersFeature — 오답 노트 목록 + 삭제 상태 관리.
/// FavoritesView 세그먼트 "오답 노트" 탭에서 사용.
@Observable
@MainActor
final class WrongAnswersFeature {

    // MARK: - Phase

    enum Phase: Equatable {
        case idle
        case loading
        case done
        case failed(String)
    }

    // MARK: - State

    private(set) var phase: Phase = .idle
    private(set) var items: [WrongAnswer] = []
    private(set) var deleteError: String? = nil

    // MARK: - Dependencies

    private let wrongAnswer: any WrongAnswerPort

    // MARK: - Init

    init(wrongAnswer: any WrongAnswerPort) {
        self.wrongAnswer = wrongAnswer
    }

    // MARK: - Actions

    /// GET /me/quiz/wrong-answers → items 갱신 (created_at DESC).
    func load() async {
        phase = .loading
        do {
            items = try await wrongAnswer.list()
            phase = .done
        } catch {
            phase = .failed("오답 노트를 불러오지 못했습니다.")
        }
    }

    /// DELETE /me/quiz/wrong-answers/{id} → items에서 제거.
    /// 실패 시 items 유지 + deleteError 설정.
    func delete(id: UUID) async {
        deleteError = nil
        do {
            try await wrongAnswer.delete(id: id)
            items = items.filter { $0.id != id }
        } catch {
            deleteError = "삭제에 실패했습니다. 다시 시도해주세요."
        }
    }

    /// 삭제 에러 초기화.
    func clearDeleteError() {
        deleteError = nil
    }
}
