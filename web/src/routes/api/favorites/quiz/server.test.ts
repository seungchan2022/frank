// POST /api/favorites/quiz 단위 테스트.
// 즐겨찾기 기사 퀴즈 생성 프록시가 올바르게 동작하는지 검증.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { POST } from './+server';

// fetch mock
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

function makeEvent(
	body: unknown,
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

const mockQuizResponse = {
	questions: [
		{
			question: '테스트 질문?',
			options: ['A', 'B', 'C', 'D'],
			answer_index: 0,
			explanation: '테스트 해설'
		}
	]
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('POST /api/favorites/quiz', () => {
	it('세션 없으면 401 반환', async () => {
		const event = makeEvent({ url: 'https://example.com/article' }, null);
		const response = await POST(event as Parameters<typeof POST>[0]);
		const data = await response.json();

		expect(response.status).toBe(401);
		expect(data.error).toBe('Unauthorized');
	});

	it('정상 요청 시 퀴즈 반환', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: true,
			json: async () => mockQuizResponse
		});

		const event = makeEvent({ url: 'https://example.com/article' });
		const response = await POST(event as Parameters<typeof POST>[0]);
		const data = await response.json();

		expect(response.status).toBe(200);
		expect(mockFetch).toHaveBeenCalledTimes(1);

		const [calledUrl, calledOptions] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(calledUrl).toContain('/api/me/favorites/quiz');
		expect(calledOptions.headers).toMatchObject({
			Authorization: 'Bearer test.jwt.token'
		});
		expect(data.questions).toHaveLength(1);
		expect(data.questions[0].question).toBe('테스트 질문?');
	});

	it('즐겨찾기 없는 기사 404 처리', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: false,
			status: 404,
			json: async () => ({ error: '즐겨찾기에 없는 기사입니다.' })
		});

		const event = makeEvent({ url: 'https://not-favorited.com' });
		const response = await POST(event as Parameters<typeof POST>[0]);
		const data = await response.json();

		expect(response.status).toBe(404);
		expect(data.error).toBeDefined();
	});

	it('LLM 실패 시 503 처리', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: false,
			status: 503,
			json: async () => ({ error: '퀴즈 생성 실패' })
		});

		const event = makeEvent({ url: 'https://example.com/article' });
		const response = await POST(event as Parameters<typeof POST>[0]);
		const data = await response.json();

		expect(response.status).toBe(503);
		expect(data.error).toBeDefined();
	});

	it('잘못된 JSON body 시 400 반환', async () => {
		const event = {
			request: {
				json: vi.fn().mockRejectedValue(new Error('Invalid JSON'))
			},
			locals: {
				safeGetSession: vi
					.fn()
					.mockResolvedValue({ session: { access_token: 'token' }, user: { id: 'u1' } })
			}
		};
		const response = await POST(event as Parameters<typeof POST>[0]);
		const data = await response.json();

		expect(response.status).toBe(400);
		expect(data.error).toBeDefined();
	});

	it('네트워크 에러 시 500 JSON 에러 반환', async () => {
		mockFetch.mockRejectedValueOnce(new Error('네트워크 오류'));

		const event = makeEvent({ url: 'https://example.com/article' });
		const response = await POST(event as Parameters<typeof POST>[0]);
		const data = await response.json();

		expect(response.status).toBe(500);
		expect(data.error).toBe('네트워크 오류');
	});
});
