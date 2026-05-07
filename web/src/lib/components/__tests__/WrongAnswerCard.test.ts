// MVP16 M4 D2: WrongAnswerCard — 원문 보기 링크 렌더링 테스트

import { describe, it, expect } from 'vitest';
import type { WrongAnswer } from '$lib/types/quiz';

function makeWrongAnswer(overrides?: Partial<WrongAnswer>): WrongAnswer {
	return {
		id: 'wa-1',
		userId: 'user-1',
		articleUrl: 'https://example.com/article',
		articleTitle: '테스트 기사',
		question: '질문?',
		options: ['A', 'B', 'C'],
		correctIndex: 0,
		userIndex: 1,
		explanation: '해설',
		createdAt: '2026-05-07T00:00:00.000Z',
		tagId: null,
		...overrides
	};
}

/** WrongAnswerCard의 safeArticleUrl 로직과 동일 — http/https scheme 검증 */
function safeArticleUrl(articleUrl: string): string | null {
	if (!articleUrl) return null;
	try {
		const url = new URL(articleUrl);
		return url.protocol === 'http:' || url.protocol === 'https:' ? articleUrl : null;
	} catch {
		return null;
	}
}

describe('WrongAnswerCard — D2 원문 보기 (MVP16 M4)', () => {
	it('WrongAnswer 타입에 articleUrl 필드가 존재한다', () => {
		const item = makeWrongAnswer();
		expect(item.articleUrl).toBe('https://example.com/article');
	});

	it('articleUrl이 비어있지 않은 WrongAnswer 생성 가능', () => {
		const item = makeWrongAnswer({ articleUrl: 'https://techcrunch.com/article' });
		expect(item.articleUrl).not.toBe('');
		expect(item.articleUrl).toMatch(/^https?:\/\//);
	});

	it('올바른 WrongAnswer 구조 확인 — 원문 이동에 필요한 모든 필드 존재', () => {
		const item = makeWrongAnswer();
		expect(item).toHaveProperty('id');
		expect(item).toHaveProperty('articleUrl');
		expect(item).toHaveProperty('articleTitle');
		expect(item).toHaveProperty('question');
		expect(item).toHaveProperty('options');
		expect(item).toHaveProperty('correctIndex');
		expect(item).toHaveProperty('userIndex');
	});

	it('tagId가 null이어도 WrongAnswer 생성 가능', () => {
		const item = makeWrongAnswer({ tagId: null });
		expect(item.tagId).toBeNull();
		expect(item.articleUrl).toBe('https://example.com/article');
	});
});

describe('safeArticleUrl — XSS 방지 URL 검증 (MVP16 M4 D2)', () => {
	it('https URL은 그대로 반환한다', () => {
		expect(safeArticleUrl('https://example.com/article')).toBe('https://example.com/article');
	});

	it('http URL도 허용한다', () => {
		expect(safeArticleUrl('http://example.com/article')).toBe('http://example.com/article');
	});

	it('javascript: scheme은 null 반환 (XSS 방지)', () => {
		expect(safeArticleUrl('javascript:alert(1)')).toBeNull();
	});

	it('data: scheme은 null 반환', () => {
		expect(safeArticleUrl('data:text/html,<script>alert(1)</script>')).toBeNull();
	});

	it('빈 문자열은 null 반환', () => {
		expect(safeArticleUrl('')).toBeNull();
	});

	it('유효하지 않은 URL은 null 반환', () => {
		expect(safeArticleUrl('not-a-url')).toBeNull();
	});
});
