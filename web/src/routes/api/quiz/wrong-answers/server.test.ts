// GET/POST /api/quiz/wrong-answers 단위 테스트.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { GET, POST } from './+server';

const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

function makeEvent(
	body: unknown = null,
	session: { access_token: string } | null = { access_token: 'test.jwt.token' }
) {
	return {
		request: {
			json: vi.fn().mockResolvedValue(body)
		},
		locals: {
			safeGetSession: vi.fn().mockResolvedValue({ session, user: session ? { id: 'u1' } : null })
		}
	};
}

const mockWrongAnswer = {
	id: 'wa-1',
	user_id: 'u1',
	article_url: 'https://example.com',
	article_title: '테스트 기사',
	question: '테스트 질문?',
	options: ['A', 'B', 'C', 'D'],
	correct_index: 0,
	user_index: 1,
	explanation: '해설',
	created_at: '2026-04-12T00:00:00Z'
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('GET /api/quiz/wrong-answers', () => {
	it('세션 없으면 401 반환', async () => {
		const event = makeEvent(null, null);
		const response = await GET(event as Parameters<typeof GET>[0]);
		const data = await response.json();

		expect(response.status).toBe(401);
		expect(data.error).toBe('Unauthorized');
	});

	it('정상 요청 시 오답 목록 반환', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: true,
			json: async () => [mockWrongAnswer]
		});

		const event = makeEvent();
		const response = await GET(event as Parameters<typeof GET>[0]);
		const data = await response.json();

		expect(response.status).toBe(200);
		expect(data).toHaveLength(1);
		expect(data[0].id).toBe('wa-1');
	});

	it('업스트림 에러 시 에러 상태 반환', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: false,
			status: 500,
			json: async () => ({ error: 'Internal error' })
		});

		const event = makeEvent();
		const response = await GET(event as Parameters<typeof GET>[0]);

		expect(response.status).toBe(500);
	});
});

describe('POST /api/quiz/wrong-answers', () => {
	it('세션 없으면 401 반환', async () => {
		const event = makeEvent({}, null);
		const response = await POST(event as Parameters<typeof POST>[0]);
		const data = await response.json();

		expect(response.status).toBe(401);
		expect(data.error).toBe('Unauthorized');
	});

	it('정상 요청 시 오답 저장 후 201 반환', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: true,
			json: async () => mockWrongAnswer
		});

		const event = makeEvent({
			article_url: 'https://example.com',
			article_title: '테스트 기사',
			question: '테스트 질문?',
			options: ['A', 'B', 'C', 'D'],
			correct_index: 0,
			user_index: 1,
			explanation: '해설'
		});
		const response = await POST(event as Parameters<typeof POST>[0]);
		const data = await response.json();

		expect(response.status).toBe(201);
		expect(data.id).toBe('wa-1');
	});

	it('잘못된 JSON body 시 400 반환', async () => {
		const event = {
			request: { json: vi.fn().mockRejectedValue(new Error('Invalid JSON')) },
			locals: {
				safeGetSession: vi
					.fn()
					.mockResolvedValue({ session: { access_token: 'token' }, user: { id: 'u1' } })
			}
		};
		const response = await POST(event as Parameters<typeof POST>[0]);
		expect(response.status).toBe(400);
	});
});
