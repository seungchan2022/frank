import Testing
import Foundation
@testable import Frank

/// MVP9 M2: QuizView 세션 오답 누적 로직 테스트.
///
/// QuizView 자체는 SwiftUI View라 직접 단위테스트가 어렵다.
/// 세션 오답 누적 로직의 핵심 불변식을 도메인 레벨에서 검증한다.
///   - SessionWrongAnswer 내부 구조체 로직
///   - 오답 누적 시나리오
///   - 새 퀴즈 시작 시 리셋 시나리오
@Suite("QuizView SessionWrongAnswer 로직 Tests — MVP9 M2")
struct QuizViewSessionTests {

    // MARK: - Helpers

    private func makeQuestion(
        question: String = "질문?",
        options: [String] = ["A", "B", "C"],
        answerIndex: Int = 0,
        explanation: String = "해설"
    ) -> QuizQuestion {
        QuizQuestion(
            question: question,
            options: options,
            answerIndex: answerIndex,
            explanation: explanation
        )
    }

    // MARK: - SessionWrongAnswerAccumulator 로직 테스트

    @Test("오답 선택 시 누적 카운트 증가")
    func wrongAnswer_accumulates_count() {
        var acc = SessionWrongAnswerAccumulator()

        let q1 = makeQuestion(question: "Q1", answerIndex: 0)
        acc.record(question: q1, userIndex: 1)  // 오답

        #expect(acc.items.count == 1)
    }

    @Test("정답 선택 시 누적 카운트 변화 없음 - 오답만 기록")
    func correct_answer_does_not_accumulate() {
        var acc = SessionWrongAnswerAccumulator()

        let q1 = makeQuestion(question: "Q1", answerIndex: 0)
        acc.record(question: q1, userIndex: 0)  // 정답

        // 정답이므로 기록 안 됨
        #expect(acc.items.count == 0)
    }

    @Test("오답 3개 연속 누적 시 모두 저장됨")
    func multiple_wrong_answers_accumulate() {
        var acc = SessionWrongAnswerAccumulator()

        let q1 = makeQuestion(question: "Q1", answerIndex: 0)
        let q2 = makeQuestion(question: "Q2", answerIndex: 1)
        let q3 = makeQuestion(question: "Q3", answerIndex: 2)

        acc.record(question: q1, userIndex: 1)
        acc.record(question: q2, userIndex: 0)
        acc.record(question: q3, userIndex: 1)

        #expect(acc.items.count == 3)
    }

    @Test("오답 누적 후 reset 시 빈 배열")
    func reset_clears_accumulated_items() {
        var acc = SessionWrongAnswerAccumulator()

        let q = makeQuestion(question: "Q1", answerIndex: 0)
        acc.record(question: q, userIndex: 1)
        #expect(acc.items.count == 1)

        acc.reset()
        #expect(acc.items.isEmpty)
    }

    @Test("오답 아이템의 question 필드가 올바르게 저장됨")
    func wrong_answer_stores_correct_question() {
        var acc = SessionWrongAnswerAccumulator()
        let q = makeQuestion(question: "오답 질문입니까?", options: ["A", "B"], answerIndex: 0)

        acc.record(question: q, userIndex: 1)

        #expect(acc.items.first?.question.question == "오답 질문입니까?")
    }

    @Test("오답 아이템의 userIndex 필드가 올바르게 저장됨")
    func wrong_answer_stores_correct_user_index() {
        var acc = SessionWrongAnswerAccumulator()
        let q = makeQuestion(question: "Q?", options: ["A", "B", "C"], answerIndex: 0)

        acc.record(question: q, userIndex: 2)

        #expect(acc.items.first?.userIndex == 2)
    }

    @Test("정답+오답 혼합 시 오답만 저장됨")
    func mixed_answers_only_stores_wrong() {
        var acc = SessionWrongAnswerAccumulator()

        let q1 = makeQuestion(question: "Q1", answerIndex: 0)
        let q2 = makeQuestion(question: "Q2", answerIndex: 1)
        let q3 = makeQuestion(question: "Q3", answerIndex: 0)

        acc.record(question: q1, userIndex: 0)  // 정답
        acc.record(question: q2, userIndex: 0)  // 오답
        acc.record(question: q3, userIndex: 0)  // 정답

        #expect(acc.items.count == 1)
        #expect(acc.items.first?.question.question == "Q2")
    }
}
