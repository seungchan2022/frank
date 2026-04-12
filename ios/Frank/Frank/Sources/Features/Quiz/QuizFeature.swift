import Foundation
import Observation

/// MVP7 M4: QuizFeature — 퀴즈 생성 + 진행 상태 관리.
///
/// ArticleDetailView에서 퀴즈 버튼 탭 시 generateQuiz()를 호출.
/// 결과가 로드되면 questions에 저장되고 QuizView가 표시된다.
///
/// MVP8 M3: 오답 저장(fire-and-forget) + 퀴즈 완료 마킹 추가.
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
    private let wrongAnswer: any WrongAnswerPort
    private let favorites: any FavoritesPort

    // MARK: - Context (퀴즈 진행 중 기사 정보)

    private var articleUrl: String = ""
    private var articleTitle: String = ""

    // MARK: - Init

    init(quiz: any QuizPort, wrongAnswer: any WrongAnswerPort, favorites: any FavoritesPort) {
        self.quiz = quiz
        self.wrongAnswer = wrongAnswer
        self.favorites = favorites
    }

    // MARK: - Actions

    /// 퀴즈 생성 요청.
    /// - Parameters:
    ///   - url: 즐겨찾기한 기사 URL 문자열
    ///   - title: 기사 제목 (오답 저장용)
    func generateQuiz(url: String, title: String = "") async {
        guard phase != .loading else { return }
        articleUrl = url
        articleTitle = title
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

    /// 오답 발생 시 fire-and-forget 저장.
    /// QuizView의 confirm 시점에 호출.
    func saveWrongAnswer(question: QuizQuestion, userIndex: Int) {
        guard !articleUrl.isEmpty else { return }
        let params = SaveWrongAnswerParams(
            articleUrl: articleUrl,
            articleTitle: articleTitle,
            question: question.question,
            options: question.options,
            correctIndex: question.answerIndex,
            userIndex: userIndex,
            explanation: question.explanation
        )
        Task {
            try? await wrongAnswer.save(params: params)
        }
    }

    /// 퀴즈 완료(마지막 문제 통과) 시 호출.
    /// 중복 방지는 QuizView의 quizCompletedMarked 플래그에서 처리.
    func markQuizCompleted() {
        guard !articleUrl.isEmpty else { return }
        let url = articleUrl
        Task {
            try? await favorites.markQuizCompleted(url: url)
        }
    }

    /// 퀴즈 완료 후 상태 초기화.
    func reset() {
        phase = .idle
        questions = []
        articleUrl = ""
        articleTitle = ""
    }
}
