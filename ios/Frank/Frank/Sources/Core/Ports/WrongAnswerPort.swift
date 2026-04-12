import Foundation

/// MVP8 M3: 오답 아카이빙 포트.
/// GET/POST/DELETE /api/me/quiz/wrong-answers
protocol WrongAnswerPort: Sendable {
    /// 오답 1건 저장.
    func save(params: SaveWrongAnswerParams) async throws -> WrongAnswer
    /// 오답 목록 조회 (created_at DESC).
    func list() async throws -> [WrongAnswer]
    /// 오답 1건 삭제.
    func delete(id: UUID) async throws
}
