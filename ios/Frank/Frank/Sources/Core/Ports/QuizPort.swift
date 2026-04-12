import Foundation

/// MVP7 M4: 퀴즈 생성 포트.
/// POST /api/me/favorites/quiz
protocol QuizPort: Sendable {
    /// 즐겨찾기 기사 URL로 퀴즈 3문제를 생성한다.
    /// - Parameter url: 즐겨찾기한 기사 URL 문자열
    /// - Returns: QuizQuestion 배열
    /// - Throws: APIQuizError.notFound (즐겨찾기 미존재), APIQuizError.serviceUnavailable (LLM 실패)
    func generateQuiz(url: String) async throws -> [QuizQuestion]
}

/// 퀴즈 문제 도메인 모델.
struct QuizQuestion: Decodable, Equatable, Sendable {
    let question: String
    let options: [String]
    let answerIndex: Int
    let explanation: String

    enum CodingKeys: String, CodingKey {
        case question, options, explanation
        case answerIndex = "answer_index"
    }
}
