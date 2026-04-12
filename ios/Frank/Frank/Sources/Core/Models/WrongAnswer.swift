import Foundation

/// MVP8 M3: 오답 아카이빙 모델.
/// 서버 quiz_wrong_answers 테이블과 1:1 대응.
struct WrongAnswer: Codable, Identifiable, Equatable, Sendable {
    let id: UUID
    let userId: UUID
    let articleUrl: String
    let articleTitle: String
    let question: String
    let options: [String]
    let correctIndex: Int
    let userIndex: Int
    let explanation: String?
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id
        case userId = "user_id"
        case articleUrl = "article_url"
        case articleTitle = "article_title"
        case question
        case options
        case correctIndex = "correct_index"
        case userIndex = "user_index"
        case explanation
        case createdAt = "created_at"
    }
}

/// POST /me/quiz/wrong-answers 요청 바디.
struct SaveWrongAnswerParams: Encodable {
    let articleUrl: String
    let articleTitle: String
    let question: String
    let options: [String]
    let correctIndex: Int
    let userIndex: Int
    let explanation: String?

    enum CodingKeys: String, CodingKey {
        case articleUrl = "article_url"
        case articleTitle = "article_title"
        case question
        case options
        case correctIndex = "correct_index"
        case userIndex = "user_index"
        case explanation
    }
}
