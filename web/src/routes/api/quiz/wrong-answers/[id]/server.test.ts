// DELETE /api/quiz/wrong-answers/[id] 단위 테스트.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { DELETE } from './+server';

const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

function makeEvent(
	id = 'wa-1',
	session: { access_token: string } | null = { access_token: 'test.jwt.token' }
) {
	return {
		params: { id },
		locals: {
			safeGetSession: vi.fn().mockResolvedValue({ session, user: session ? { id: 'u1' } : null })
		}
	};
}

beforeEach(() => {
	vi.clearAllMocks();
});

describe('DELETE /api/quiz/wrong-answers/[id]', () => {
	it('세션 없으면 401 반환', async () => {
		const event = makeEvent('wa-1', null);
		const response = await DELETE(event as Parameters<typeof DELETE>[0]);
		const data = await response.json();

		expect(response.status).toBe(401);
		expect(data.error).toBe('Unauthorized');
	});

	it('정상 삭제 시 204 반환', async () => {
		mockFetch.mockResolvedValueOnce({ ok: true });

		const event = makeEvent('wa-1');
		const response = await DELETE(event as Parameters<typeof DELETE>[0]);

		expect(response.status).toBe(204);
		const [calledUrl] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(calledUrl).toContain('wa-1');
	});

	it('업스트림 404 시 404 반환', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: false,
			status: 404,
			json: async () => ({ error: 'Not found' })
		});

		const event = makeEvent('nonexistent');
		const response = await DELETE(event as Parameters<typeof DELETE>[0]);

		expect(response.status).toBe(404);
	});

	it('네트워크 에러 시 500 JSON 에러 반환', async () => {
		mockFetch.mockRejectedValueOnce(new Error('network error'));

		const event = makeEvent('wa-1');
		const response = await DELETE(event as Parameters<typeof DELETE>[0]);
		const data = await response.json();

		expect(response.status).toBe(500);
		expect(data.error).toBe('network error');
	});
});
