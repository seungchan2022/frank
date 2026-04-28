import Foundation
@testable import Frank

final class MockWrongAnswerPort: WrongAnswerPort, @unchecked Sendable {
    private var store: [UUID: WrongAnswer] = [:]
    private var insertOrder: [UUID] = []

    var saveCallCount = 0
    var listCallCount = 0
    var deleteCallCount = 0
    var shouldFail = false

    func save(params: SaveWrongAnswerParams) async throws -> WrongAnswer {
        saveCallCount += 1
        if shouldFail { throw MockWrongAnswerError.generic }
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
            createdAt: Date(),
            tagId: params.tagId
        )
        store[wa.id] = wa
        insertOrder.append(wa.id)
        return wa
    }

    func list() async throws -> [WrongAnswer] {
        listCallCount += 1
        if shouldFail { throw MockWrongAnswerError.generic }
        return insertOrder.reversed().compactMap { store[$0] }
    }

    func delete(id: UUID) async throws {
        deleteCallCount += 1
        if shouldFail { throw MockWrongAnswerError.generic }
        store.removeValue(forKey: id)
        insertOrder.removeAll { $0 == id }
    }
}

enum MockWrongAnswerError: Error {
    case generic
}
