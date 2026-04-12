import Foundation

/// In-memory WrongAnswerPort 구현 — FRANK_USE_MOCK=1 모드 전용.
final class MockWrongAnswerAdapter: WrongAnswerPort, @unchecked Sendable {
    private var store: [UUID: WrongAnswer] = [:]
    private var insertOrder: [UUID] = []

    func save(params: SaveWrongAnswerParams) async throws -> WrongAnswer {
        let wa = WrongAnswer(
            id: UUID(),
            userId: UUID(),
            articleUrl: params.articleUrl,
            articleTitle: params.articleTitle,
            question: params.question,
            options: params.options,
            correctIndex: params.correctIndex,
            userIndex: params.userIndex,
            explanation: params.explanation,
            createdAt: Date()
        )
        store[wa.id] = wa
        insertOrder.append(wa.id)
        return wa
    }

    func list() async throws -> [WrongAnswer] {
        return insertOrder.reversed().compactMap { store[$0] }
    }

    func delete(id: UUID) async throws {
        store.removeValue(forKey: id)
        insertOrder.removeAll { $0 == id }
    }
}
