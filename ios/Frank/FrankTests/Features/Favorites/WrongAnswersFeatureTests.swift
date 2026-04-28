import Testing
import Foundation
@testable import Frank

@Suite("WrongAnswersFeature Tests — MVP8 M3")
@MainActor
struct WrongAnswersFeatureTests {

    private func makeSUT(
        port: MockWrongAnswerPort = MockWrongAnswerPort()
    ) -> (WrongAnswersFeature, MockWrongAnswerPort) {
        let feature = WrongAnswersFeature(wrongAnswer: port)
        return (feature, port)
    }

    // MARK: - 초기 상태

    @Test("초기 상태: phase idle, items 빈 배열")
    func initialState() {
        let (sut, _) = makeSUT()
        #expect(sut.phase == .idle)
        #expect(sut.items.isEmpty)
    }

    // MARK: - 목록 로딩

    @Test("load 성공: phase done, items 채워짐")
    func load_success() async {
        let port = MockWrongAnswerPort()
        // 먼저 오답 저장
        _ = try? await port.save(params: SaveWrongAnswerParams(
            articleUrl: "https://example.com",
            articleTitle: "테스트 기사",
            question: "질문?",
            options: ["A", "B"],
            correctIndex: 0,
            userIndex: 1,
            explanation: "해설",
            tagId: nil
        ))
        let (sut, _) = makeSUT(port: port)

        await sut.load()

        #expect(sut.phase == .done)
        #expect(sut.items.count == 1)
    }

    @Test("load 실패: phase failed")
    func load_failure() async {
        let port = MockWrongAnswerPort()
        port.shouldFail = true
        let (sut, _) = makeSUT(port: port)

        await sut.load()

        if case .failed = sut.phase {
            // expected
        } else {
            Issue.record("Expected failed phase")
        }
    }

    @Test("load: port.list 1회 호출")
    func load_calls_port_once() async {
        let port = MockWrongAnswerPort()
        let (sut, _) = makeSUT(port: port)

        await sut.load()

        #expect(port.listCallCount == 1)
    }

    // MARK: - 삭제

    @Test("delete 성공: items에서 제거")
    func delete_removes_item() async {
        let port = MockWrongAnswerPort()
        _ = try? await port.save(params: SaveWrongAnswerParams(
            articleUrl: "https://example.com",
            articleTitle: "테스트 기사",
            question: "질문?",
            options: ["A", "B"],
            correctIndex: 0,
            userIndex: 1,
            explanation: "해설",
            tagId: nil
        ))
        let (sut, _) = makeSUT(port: port)
        await sut.load()
        #expect(sut.items.count == 1)

        let id = sut.items[0].id
        await sut.delete(id: id)

        #expect(sut.items.isEmpty)
    }

    @Test("delete 실패: items 변경 없음, deleteError 설정")
    func delete_failure_keeps_items() async {
        let port = MockWrongAnswerPort()
        _ = try? await port.save(params: SaveWrongAnswerParams(
            articleUrl: "https://example.com",
            articleTitle: "테스트 기사",
            question: "질문?",
            options: ["A", "B"],
            correctIndex: 0,
            userIndex: 1,
            explanation: "해설",
            tagId: nil
        ))
        let (sut, _) = makeSUT(port: port)
        await sut.load()
        #expect(sut.items.count == 1)

        port.shouldFail = true
        let id = sut.items[0].id
        await sut.delete(id: id)

        #expect(sut.items.count == 1)
        #expect(sut.deleteError != nil)
    }

    @Test("clearDeleteError: deleteError nil로 초기화")
    func clearDeleteError() async {
        let port = MockWrongAnswerPort()
        _ = try? await port.save(params: SaveWrongAnswerParams(
            articleUrl: "https://example.com",
            articleTitle: "테스트 기사",
            question: "질문?",
            options: ["A", "B"],
            correctIndex: 0,
            userIndex: 1,
            explanation: "해설",
            tagId: nil
        ))
        let (sut, _) = makeSUT(port: port)
        await sut.load()
        port.shouldFail = true
        await sut.delete(id: sut.items[0].id)
        #expect(sut.deleteError != nil)

        sut.clearDeleteError()

        #expect(sut.deleteError == nil)
    }
}
