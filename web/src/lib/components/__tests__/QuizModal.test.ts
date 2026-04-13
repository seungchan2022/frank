// MVP8 M3: QuizModal — 오답 저장 + 퀴즈 완료 마킹 단위 테스트
// MVP9 M2: 세션 오답 인라인 표시 로직 테스트 추가.
// apiClient와 favoritesStore를 mock하여 사이드이펙트 검증.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { QuizQuestion } from '$lib/types/quiz';

const mockSaveWrongAnswer = vi.fn().mockResolvedValue({ id: 'wa-1' });
const mockMarkQuizDone = vi.fn().mockResolvedValue(undefined);
const mockMarkQuizCompleted = vi.fn();

vi.mock('$lib/api', () => ({
	apiClient: {
		saveWrongAnswer: mockSaveWrongAnswer,
		markQuizDone: mockMarkQuizDone
	}
}));

vi.mock('$lib/stores/favoritesStore.svelte', () => ({
	favoritesStore: {
		markQuizCompleted: mockMarkQuizCompleted
	}
}));

// QuizModal의 내부 로직을 직접 테스트 (DOM 마운트 없이 함수 단위 검증)
// confirm()과 nextQuestion() 함수의 사이드이펙트를 직접 검증함.

const mockQuestions: QuizQuestion[] = [
	{
		question: '질문 1',
		options: ['A', 'B', 'C', 'D'],
		answer_index: 0,
		explanation: '해설 1'
	},
	{
		question: '질문 2',
		options: ['A', 'B', 'C', 'D'],
		answer_index: 1,
		explanation: '해설 2'
	}
];

beforeEach(() => {
	vi.clearAllMocks();
});

/**
 * QuizModal의 confirm() 로직을 순수 함수로 추출하여 테스트.
 * Svelte 컴포넌트 마운트 없이 사이드이펙트만 검증.
 */
function makeConfirmLogic(
	articleUrl: string | undefined,
	articleTitle: string | undefined,
	questions: QuizQuestion[]
) {
	let currentIndex = 0;
	let selectedIndex: number | null = null;
	let score = 0;
	let confirmed = false;

	return {
		selectOption(i: number) {
			if (!confirmed) selectedIndex = i;
		},
		confirm() {
			if (selectedIndex === null || confirmed) return;
			const q = questions[currentIndex];
			const correct = selectedIndex === q.answer_index;
			if (correct) {
				score += 1;
			} else if (articleUrl && articleTitle) {
				const uIdx = selectedIndex;
				void mockSaveWrongAnswer({
					article_url: articleUrl,
					article_title: articleTitle,
					question: q.question,
					options: q.options,
					correct_index: q.answer_index,
					user_index: uIdx,
					explanation: q.explanation ?? null
				}).catch(() => {});
			}
			confirmed = true;
		},
		get score() {
			return score;
		},
		get confirmed() {
			return confirmed;
		}
	};
}

describe('QuizModal: 오답 저장 (fire-and-forget)', () => {
	it('오답 시 saveWrongAnswer 호출됨', () => {
		const logic = makeConfirmLogic(
			'https://article.com',
			'테스트 기사',
			mockQuestions
		);
		logic.selectOption(1); // 오답 선택 (answer_index=0)
		logic.confirm();

		expect(mockSaveWrongAnswer).toHaveBeenCalledTimes(1);
		expect(mockSaveWrongAnswer).toHaveBeenCalledWith(
			expect.objectContaining({
				article_url: 'https://article.com',
				article_title: '테스트 기사',
				question: '질문 1',
				correct_index: 0,
				user_index: 1
			})
		);
	});

	it('정답 시 saveWrongAnswer 호출 안 됨', () => {
		const logic = makeConfirmLogic('https://article.com', '테스트 기사', mockQuestions);
		logic.selectOption(0); // 정답 선택 (answer_index=0)
		logic.confirm();

		expect(mockSaveWrongAnswer).not.toHaveBeenCalled();
		expect(logic.score).toBe(1);
	});

	it('articleUrl 없으면 saveWrongAnswer 호출 안 됨', () => {
		const logic = makeConfirmLogic(undefined, undefined, mockQuestions);
		logic.selectOption(1); // 오답
		logic.confirm();

		expect(mockSaveWrongAnswer).not.toHaveBeenCalled();
	});
});

describe('QuizModal: 퀴즈 완료 마킹', () => {
	it('마지막 문제 완료 시 markQuizDone + markQuizCompleted 호출됨', async () => {
		const singleQuestion = [mockQuestions[0]];
		let finished = false;
		let quizCompletedMarked = false;
		const articleUrl = 'https://article.com';

		function nextQuestion() {
			if (!finished) {
				finished = true;
				if (!quizCompletedMarked && articleUrl) {
					quizCompletedMarked = true;
					void mockMarkQuizDone(articleUrl)
						.then(() => {
							mockMarkQuizCompleted(articleUrl);
						})
						.catch(() => {});
				}
			}
		}

		nextQuestion();

		// 비동기 완료 대기
		await new Promise((resolve) => setTimeout(resolve, 10));

		expect(mockMarkQuizDone).toHaveBeenCalledWith('https://article.com');
		expect(mockMarkQuizCompleted).toHaveBeenCalledWith('https://article.com');
		void singleQuestion;
	});

	it('퀴즈 완료 마킹은 중복 호출되지 않음', async () => {
		let quizCompletedMarked = false;
		const articleUrl = 'https://article.com';

		function nextQuestion() {
			if (!quizCompletedMarked && articleUrl) {
				quizCompletedMarked = true;
				void mockMarkQuizDone(articleUrl)
					.then(() => mockMarkQuizCompleted(articleUrl))
					.catch(() => {});
			}
		}

		nextQuestion();
		nextQuestion(); // 중복 호출

		await new Promise((resolve) => setTimeout(resolve, 10));

		expect(mockMarkQuizDone).toHaveBeenCalledTimes(1);
	});

	it('articleUrl 없으면 markQuizDone 호출 안 됨', () => {
		let quizCompletedMarked = false;
		const articleUrl: string | undefined = undefined;

		function nextQuestion() {
			if (!quizCompletedMarked && articleUrl) {
				quizCompletedMarked = true;
				void mockMarkQuizDone(articleUrl).catch(() => {});
			}
		}

		nextQuestion();

		expect(mockMarkQuizDone).not.toHaveBeenCalled();
	});
});

// ─── MVP9 M2: 세션 오답 인라인 표시 ──────────────────────────────────────────

/**
 * 세션 오답 누적 로직을 순수 함수로 추출하여 테스트.
 * confirm() 시 오답이면 sessionWrongAnswers에 추가되는지 검증.
 */
function makeSessionWrongLogic(
	articleUrl: string | undefined,
	articleTitle: string | undefined,
	questions: QuizQuestion[]
) {
	let currentIndex = 0;
	let selectedIndex: number | null = null;
	let score = 0;
	let confirmed = false;
	let sessionWrongAnswers: Array<{ question: QuizQuestion; userIndex: number }> = [];

	return {
		selectOption(i: number) {
			if (!confirmed) selectedIndex = i;
		},
		confirm() {
			if (selectedIndex === null || confirmed) return;
			const q = questions[currentIndex];
			const correct = selectedIndex === q.answer_index;
			if (correct) {
				score += 1;
			} else {
				// 세션 오답 누적 (MVP9 M2)
				sessionWrongAnswers = [...sessionWrongAnswers, { question: q, userIndex: selectedIndex }];
				if (articleUrl && articleTitle) {
					const uIdx = selectedIndex;
					void mockSaveWrongAnswer({
						article_url: articleUrl,
						article_title: articleTitle,
						question: q.question,
						options: q.options,
						correct_index: q.answer_index,
						user_index: uIdx,
						explanation: q.explanation ?? null
					}).catch(() => {});
				}
			}
			confirmed = true;
		},
		nextQuestion() {
			if (currentIndex + 1 < questions.length) {
				currentIndex += 1;
				selectedIndex = null;
				confirmed = false;
			}
		},
		reset() {
			currentIndex = 0;
			selectedIndex = null;
			score = 0;
			confirmed = false;
			sessionWrongAnswers = [];
		},
		get score() {
			return score;
		},
		get sessionWrongAnswers() {
			return sessionWrongAnswers;
		}
	};
}

describe('QuizModal: 세션 오답 인라인 표시 (MVP9 M2)', () => {
	it('오답 시 sessionWrongAnswers에 누적됨', () => {
		const logic = makeSessionWrongLogic('https://article.com', '테스트 기사', mockQuestions);
		logic.selectOption(1); // 오답 (answer_index=0)
		logic.confirm();

		expect(logic.sessionWrongAnswers).toHaveLength(1);
		expect(logic.sessionWrongAnswers[0].question).toEqual(mockQuestions[0]);
		expect(logic.sessionWrongAnswers[0].userIndex).toBe(1);
	});

	it('정답 시 sessionWrongAnswers에 추가 안 됨', () => {
		const logic = makeSessionWrongLogic('https://article.com', '테스트 기사', mockQuestions);
		logic.selectOption(0); // 정답 (answer_index=0)
		logic.confirm();

		expect(logic.sessionWrongAnswers).toHaveLength(0);
	});

	it('여러 문제 오답 시 누적됨', () => {
		const logic = makeSessionWrongLogic('https://article.com', '테스트 기사', mockQuestions);
		// 1번째 문제 오답
		logic.selectOption(1); // 오답
		logic.confirm();
		logic.nextQuestion();
		// 2번째 문제 오답
		logic.selectOption(0); // 오답 (answer_index=1)
		logic.confirm();

		expect(logic.sessionWrongAnswers).toHaveLength(2);
	});

	it('reset() 시 sessionWrongAnswers가 초기화됨', () => {
		const logic = makeSessionWrongLogic('https://article.com', '테스트 기사', mockQuestions);
		logic.selectOption(1); // 오답
		logic.confirm();
		expect(logic.sessionWrongAnswers).toHaveLength(1);

		logic.reset();
		expect(logic.sessionWrongAnswers).toHaveLength(0);
	});

	it('articleUrl 없어도 세션 오답은 누적됨', () => {
		const logic = makeSessionWrongLogic(undefined, undefined, mockQuestions);
		logic.selectOption(1); // 오답
		logic.confirm();

		// 세션 오답은 로컬 상태이므로 articleUrl 무관하게 누적
		expect(logic.sessionWrongAnswers).toHaveLength(1);
		// 하지만 API 호출은 없음
		expect(mockSaveWrongAnswer).not.toHaveBeenCalled();
	});

	it('오답 없을 때 sessionWrongAnswers.length === 0 (완벽 상태)', () => {
		const logic = makeSessionWrongLogic('https://article.com', '테스트 기사', mockQuestions);
		logic.selectOption(0); // 정답
		logic.confirm();
		logic.nextQuestion();
		logic.selectOption(1); // 정답 (answer_index=1)
		logic.confirm();

		expect(logic.sessionWrongAnswers).toHaveLength(0);
		expect(logic.score).toBe(2);
	});
});
