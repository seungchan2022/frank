// POST /api/favorites/quiz/done 단위 테스트.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { POST } from './+server';

const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

function makeEvent(
	body: unknown = { url: 'https://example.com' },
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

beforeEach(() => {
	vi.clearAllMocks();
});

describe('POST /api/favorites/quiz/done', () => {
	it('세션 없으면 401 반환', async () => {
		const event = makeEvent({ url: 'https://example.com' }, null);
		const response = await POST(event as Parameters<typeof POST>[0]);
		const data = await response.json();

		expect(response.status).toBe(401);
		expect(data.error).toBe('Unauthorized');
	});

	it('정상 요청 시 204 반환', async () => {
		mockFetch.mockResolvedValueOnce({ ok: true });

		const event = makeEvent({ url: 'https://example.com' });
		const response = await POST(event as Parameters<typeof POST>[0]);

		expect(response.status).toBe(204);
		const [calledUrl, calledOptions] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(calledUrl).toContain('/api/me/favorites/quiz/done');
		expect(calledOptions.headers).toMatchObject({
			Authorization: 'Bearer test.jwt.token'
		});
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

	it('업스트림 에러 시 에러 상태 반환', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: false,
			status: 404,
			json: async () => ({ error: 'Not found' })
		});

		const event = makeEvent({ url: 'https://example.com' });
		const response = await POST(event as Parameters<typeof POST>[0]);

		expect(response.status).toBe(404);
	});

	it('네트워크 에러 시 500 JSON 에러 반환', async () => {
		mockFetch.mockRejectedValueOnce(new Error('network failure'));

		const event = makeEvent();
		const response = await POST(event as Parameters<typeof POST>[0]);
		const data = await response.json();

		expect(response.status).toBe(500);
		expect(data.error).toBe('network failure');
	});
});
