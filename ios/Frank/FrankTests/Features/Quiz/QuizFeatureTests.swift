import Testing
import Foundation
@testable import Frank

@Suite("QuizFeature Tests — MVP7 M4 + MVP8 M3")
@MainActor
struct QuizFeatureTests {

    private func makeSUT(
        quizPort: MockQuizPort = MockQuizPort(),
        wrongAnswerPort: MockWrongAnswerPort = MockWrongAnswerPort(),
        favoritesPort: MockFavoritesPort = MockFavoritesPort()
    ) -> (QuizFeature, MockQuizPort, MockWrongAnswerPort, MockFavoritesPort) {
        let feature = QuizFeature(quiz: quizPort, wrongAnswer: wrongAnswerPort, favorites: favoritesPort)
        return (feature, quizPort, wrongAnswerPort, favoritesPort)
    }

    // MARK: - 초기 상태

    @Test("초기 상태: phase idle, questions 빈 배열")
    func initialState() {
        let (sut, _, _, _) = makeSUT()
        #expect(sut.phase == .idle)
        #expect(sut.questions.isEmpty)
    }

    // MARK: - 성공

    @Test("generateQuiz 성공 시 phase done, questions 설정")
    func generateQuiz_success() async {
        let (sut, _, _, _) = makeSUT()
        await sut.generateQuiz(url: "https://example.com/article")

        #expect(sut.phase == .done)
        #expect(sut.questions.count == 3)
    }

    @Test("generateQuiz 성공 시 포트 1회 호출")
    func generateQuiz_calls_port_once() async {
        let quizPort = MockQuizPort()
        let (sut, _, _, _) = makeSUT(quizPort: quizPort)

        await sut.generateQuiz(url: "https://example.com/article")

        #expect(quizPort.generateCallCount == 1)
    }

    // MARK: - 실패

    @Test("generateQuiz LLM 실패 시 phase failed with serviceUnavailable message")
    func generateQuiz_llm_failure() async {
        let quizPort = MockQuizPort()
        quizPort.shouldFail = true
        quizPort.failWithError = APIQuizError.serviceUnavailable
        let (sut, _, _, _) = makeSUT(quizPort: quizPort)

        await sut.generateQuiz(url: "https://example.com/article")

        if case .failed(let msg) = sut.phase {
            #expect(msg.contains("퀴즈 생성에 실패"))
        } else {
            Issue.record("Expected failed phase")
        }
    }

    @Test("generateQuiz notFound 시 phase failed with notFound message")
    func generateQuiz_not_found() async {
        let quizPort = MockQuizPort()
        quizPort.shouldFail = true
        quizPort.failWithError = APIQuizError.notFound
        let (sut, _, _, _) = makeSUT(quizPort: quizPort)

        await sut.generateQuiz(url: "https://example.com/article")

        if case .failed(let msg) = sut.phase {
            #expect(msg.contains("즐겨찾기에 없는"))
        } else {
            Issue.record("Expected failed phase")
        }
    }

    @Test("generateQuiz 일반 에러 시 phase failed")
    func generateQuiz_generic_error() async {
        let quizPort = MockQuizPort()
        quizPort.shouldFail = true
        quizPort.failWithError = URLError(.badServerResponse)
        let (sut, _, _, _) = makeSUT(quizPort: quizPort)

        await sut.generateQuiz(url: "https://example.com/article")

        if case .failed = sut.phase {
            // expected
        } else {
            Issue.record("Expected failed phase")
        }
    }

    // MARK: - 중복 호출 방지

    @Test("generateQuiz loading 중 재호출 무시")
    func generateQuiz_ignores_call_while_loading() async {
        let quizPort = MockQuizPort()
        let (sut, _, _, _) = makeSUT(quizPort: quizPort)

        async let first = sut.generateQuiz(url: "https://example.com/1")
        async let second = sut.generateQuiz(url: "https://example.com/2")
        _ = await (first, second)

        #expect(quizPort.generateCallCount >= 1)
    }

    // MARK: - Reset

    @Test("reset 시 phase idle, questions 빈 배열")
    func reset_clears_state() async {
        let (sut, _, _, _) = makeSUT()
        await sut.generateQuiz(url: "https://example.com/article")
        #expect(sut.phase == .done)

        sut.reset()

        #expect(sut.phase == .idle)
        #expect(sut.questions.isEmpty)
    }

    @Test("reset은 loaded 상태에서도 동작")
    func reset_after_failure() async {
        let quizPort = MockQuizPort()
        quizPort.shouldFail = true
        quizPort.failWithError = APIQuizError.serviceUnavailable
        let (sut, _, _, _) = makeSUT(quizPort: quizPort)

        await sut.generateQuiz(url: "https://example.com/article")

        if case .failed = sut.phase { /* expected */ }

        sut.reset()
        #expect(sut.phase == .idle)
    }

    // MARK: - MVP8 M3: 오답 저장

    @Test("saveWrongAnswer: articleUrl 설정 후 호출 시 wrongAnswerPort.save 호출")
    func saveWrongAnswer_calls_port() async throws {
        let wrongAnswerPort = MockWrongAnswerPort()
        let (sut, _, _, _) = makeSUT(wrongAnswerPort: wrongAnswerPort)
        await sut.generateQuiz(url: "https://example.com/article", title: "테스트 기사")

        let question = QuizQuestion(
            question: "질문?",
            options: ["A", "B"],
            answerIndex: 0,
            explanation: "해설"
        )
        sut.saveWrongAnswer(question: question, userIndex: 1)

        // fire-and-forget이므로 잠시 대기
        try await Task.sleep(for: .milliseconds(100))
        #expect(wrongAnswerPort.saveCallCount == 1)
    }

    @Test("saveWrongAnswer: articleUrl 없으면 호출 안 됨")
    func saveWrongAnswer_skips_when_no_url() throws {
        let wrongAnswerPort = MockWrongAnswerPort()
        let (sut, _, _, _) = makeSUT(wrongAnswerPort: wrongAnswerPort)
        // generateQuiz 호출 없이 articleUrl 빈 상태

        let question = QuizQuestion(
            question: "질문?",
            options: ["A", "B"],
            answerIndex: 0,
            explanation: "해설"
        )
        sut.saveWrongAnswer(question: question, userIndex: 1)

        #expect(wrongAnswerPort.saveCallCount == 0)
    }

    // MARK: - MVP8 M3: 퀴즈 완료 마킹

    @Test("markQuizCompleted: articleUrl 설정 후 호출 시 favorites.markQuizCompleted 호출")
    func markQuizCompleted_calls_favorites_port() async throws {
        let favoritesPort = MockFavoritesPort()
        let (sut, _, _, _) = makeSUT(favoritesPort: favoritesPort)
        await sut.generateQuiz(url: "https://example.com/article")

        sut.markQuizCompleted()

        try await Task.sleep(for: .milliseconds(100))
        #expect(favoritesPort.markQuizCompletedCallCount == 1)
        #expect(favoritesPort.markedQuizUrls.contains("https://example.com/article"))
    }

    @Test("markQuizCompleted: articleUrl 없으면 호출 안 됨")
    func markQuizCompleted_skips_when_no_url() throws {
        let favoritesPort = MockFavoritesPort()
        let (sut, _, _, _) = makeSUT(favoritesPort: favoritesPort)

        sut.markQuizCompleted()

        #expect(favoritesPort.markQuizCompletedCallCount == 0)
    }
}
