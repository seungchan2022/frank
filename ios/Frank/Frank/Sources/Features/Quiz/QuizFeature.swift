import Foundation
import Observation

/// MVP7 M4: QuizFeature — 퀴즈 생성 + 진행 상태 관리.
///
/// ArticleDetailView에서 퀴즈 버튼 탭 시 generateQuiz()를 호출.
/// 결과가 로드되면 questions에 저장되고 QuizView가 표시된다.
@Observable
@MainActor
final class QuizFeature {

    // MARK: - Phase

    enum Phase: Equatable {
        case idle
        case loading
        case done
        case failed(String)
    }

    // MARK: - State

    private(set) var phase: Phase = .idle
    private(set) var questions: [QuizQuestion] = []

    // MARK: - Dependencies

    private let quiz: any QuizPort

    // MARK: - Init

    init(quiz: any QuizPort) {
        self.quiz = quiz
    }

    // MARK: - Actions

    /// 퀴즈 생성 요청.
    /// - Parameter url: 즐겨찾기한 기사 URL 문자열
    func generateQuiz(url: String) async {
        guard phase != .loading else { return }
        phase = .loading
        questions = []

        do {
            let result = try await quiz.generateQuiz(url: url)
            questions = result
            phase = .done
        } catch APIQuizError.notFound {
            phase = .failed("즐겨찾기에 없는 기사입니다.")
        } catch APIQuizError.serviceUnavailable {
            phase = .failed("퀴즈 생성에 실패했습니다. 잠시 후 다시 시도해주세요.")
        } catch {
            phase = .failed("퀴즈를 불러오지 못했습니다.")
        }
    }

    /// 퀴즈 완료 후 상태 초기화.
    func reset() {
        phase = .idle
        questions = []
    }
}
