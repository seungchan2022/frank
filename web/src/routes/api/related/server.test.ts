// GET /api/related 단위 테스트.
// title + snippet 파라미터를 서버 API에 올바르게 전달하는지 검증.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { GET } from './+server';

// fetch mock
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

function makeEvent(
	params: Record<string, string>,
	session: { access_token: string } | null = { access_token: 'test.jwt.token' }
) {
	const url = new URL('http://localhost/api/related');
	for (const [key, value] of Object.entries(params)) {
		url.searchParams.set(key, value);
	}
	return {
		url,
		locals: {
			safeGetSession: vi.fn().mockResolvedValue({ session, user: session ? { id: 'u1' } : null })
		}
	};
}

beforeEach(() => {
	vi.clearAllMocks();
});

describe('GET /api/related', () => {
	it('세션 없으면 401 반환', async () => {
		const event = makeEvent({ title: '테스트', snippet: '내용' }, null);
		const response = await GET(event as Parameters<typeof GET>[0]);
		const data = await response.json();

		expect(response.status).toBe(401);
		expect(data.error).toBe('Unauthorized');
	});

	it('title + snippet으로 서버 API에 올바른 파라미터 전달', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: true,
			json: async () => [
				{ title: '연관 기사 1', url: 'https://example.com/1', snippet: '내용1', source: 'A', published_at: null, tag_id: null }
			]
		});

		const event = makeEvent({ title: 'AI 뉴스', snippet: '리드 문장' });
		const response = await GET(event as Parameters<typeof GET>[0]);
		const data = await response.json();

		expect(response.status).toBe(200);
		expect(mockFetch).toHaveBeenCalledTimes(1);

		const [calledUrl, calledOptions] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(calledUrl).toContain('/api/me/articles/related');
		expect(calledUrl).toContain('title=AI+%EB%89%B4%EC%8A%A4');
		expect(calledOptions.headers).toMatchObject({
			Authorization: 'Bearer test.jwt.token'
		});
		expect(data).toHaveLength(1);
		expect(data[0].title).toBe('연관 기사 1');
	});

	it('title만 있어도 호출됨', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: true,
			json: async () => []
		});

		const event = makeEvent({ title: 'AI 뉴스' });
		const response = await GET(event as Parameters<typeof GET>[0]);
		const data = await response.json();

		expect(response.status).toBe(200);
		expect(data).toEqual([]);
	});

	it('업스트림 에러 시 JSON 에러 응답 반환 (throw 금지)', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: false,
			status: 500,
			json: async () => ({ error: '서버 오류' })
		});

		const event = makeEvent({ title: '테스트' });
		const response = await GET(event as Parameters<typeof GET>[0]);
		const data = await response.json();

		expect(response.status).toBe(500);
		expect(data.error).toBeDefined();
	});

	it('네트워크 에러 시 500 JSON 에러 반환', async () => {
		mockFetch.mockRejectedValueOnce(new Error('네트워크 오류'));

		const event = makeEvent({ title: '테스트' });
		const response = await GET(event as Parameters<typeof GET>[0]);
		const data = await response.json();

		expect(response.status).toBe(500);
		expect(data.error).toBe('네트워크 오류');
	});
});
