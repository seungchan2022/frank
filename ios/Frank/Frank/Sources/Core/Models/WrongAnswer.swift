import Foundation

/// MVP8 M3: 오답 아카이빙 모델.
/// 서버 quiz_wrong_answers 테이블과 1:1 대응.
/// MVP13 M2: tagId 직접 포함 — favorites 브릿지 제거.
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
    let tagId: UUID?

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
        case tagId = "tag_id"
    }
}

/// POST /me/quiz/wrong-answers 요청 바디.
/// MVP13 M2: tagId 직접 전송 — 오답 저장 시 태그 연결.
struct SaveWrongAnswerParams: Encodable {
    let articleUrl: String
    let articleTitle: String
    let question: String
    let options: [String]
    let correctIndex: Int
    let userIndex: Int
    let explanation: String?
    let tagId: UUID?

    enum CodingKeys: String, CodingKey {
        case articleUrl = "article_url"
        case articleTitle = "article_title"
        case question
        case options
        case correctIndex = "correct_index"
        case userIndex = "user_index"
        case explanation
        case tagId = "tag_id"
    }
}
