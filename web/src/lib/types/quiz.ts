export interface QuizQuestion {
	question: string;
	options: string[];
	answer_index: number;
	explanation: string;
}

export interface QuizResponse {
	questions: QuizQuestion[];
}
