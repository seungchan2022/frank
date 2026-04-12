export interface QuizQuestion {
	question: string;
	options: string[];
	answer_index: number;
	explanation: string;
}

export interface QuizResponse {
	questions: QuizQuestion[];
}

/// MVP8 M3: 오답 아카이빙 타입.
/// 서버 quiz_wrong_answers 테이블과 1:1 대응.
export interface WrongAnswer {
	id: string;
	userId: string;
	articleUrl: string;
	articleTitle: string;
	question: string;
	options: string[];
	correctIndex: number;
	userIndex: number;
	explanation: string | null;
	createdAt: string;
}

/// POST /me/quiz/wrong-answers 요청 바디.
export interface SaveWrongAnswerBody {
	article_url: string;
	article_title: string;
	question: string;
	options: string[];
	correct_index: number;
	user_index: number;
	explanation: string | null;
}
