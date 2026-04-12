import Foundation
@testable import Frank

final class MockQuizPort: QuizPort, @unchecked Sendable {
    var generateCallCount = 0
    var shouldFail = false
    var failWithError: Error = APIQuizError.serviceUnavailable
    var stubbedQuestions: [QuizQuestion] = MockFixtures.quizQuestions

    func generateQuiz(url: String) async throws -> [QuizQuestion] {
        generateCallCount += 1
        if shouldFail { throw failWithError }
        return stubbedQuestions
    }
}
