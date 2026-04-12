import Foundation

/// 테스트용 MockQuizAdapter.
struct MockQuizAdapter: QuizPort {
    let questions: [QuizQuestion]
    let shouldThrow: Error?

    init(
        questions: [QuizQuestion] = MockFixtures.quizQuestions,
        shouldThrow: Error? = nil
    ) {
        self.questions = questions
        self.shouldThrow = shouldThrow
    }

    func generateQuiz(url: String) async throws -> [QuizQuestion] {
        if let error = shouldThrow {
            throw error
        }
        return questions
    }
}
