// RealApiClient HTTP 클라이언트 검증.
// fetch는 vi.fn으로 stub. 토큰은 setRealClientToken으로 결정적 주입.
// 진실의 원천: progress/260407_API_SPEC.md (엔드포인트 + 메서드 + 응답 shape)

import { describe, it, expect, vi, beforeEach, beforeAll } from 'vitest';

// VITE_RUST_API_URL은 realClient module-eval 시점에 import.meta.env로 캐싱되므로
// vi.stubEnv 후 동적 import를 통해 적용한다.
let realApiClient: typeof import('../realClient').realApiClient;
let setRealClientToken: typeof import('../realClient').setRealClientToken;

beforeAll(async () => {
	vi.stubEnv('VITE_RUST_API_URL', 'http://localhost:8081');
	vi.resetModules();
	const mod = await import('../realClient');
	realApiClient = mod.realApiClient;
	setRealClientToken = mod.setRealClientToken;
});

const FAKE_TOKEN = 'fake.jwt.token';

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' }
	});
}

function emptyResponse(status = 200): Response {
	// 204 등 null-body status는 string body 전달 시 Response constructor가 throw
	const body = status === 204 || status === 304 ? null : '';
	return new Response(body, { status });
}

beforeEach(() => {
	vi.clearAllMocks();
	setRealClientToken(FAKE_TOKEN);
	globalThis.fetch = vi.fn();
});

function lastFetchCall(): { url: string; init: RequestInit } {
	const fetchMock = globalThis.fetch as unknown as ReturnType<typeof vi.fn>;
	const call = fetchMock.mock.calls.at(-1);
	if (!call) throw new Error('fetch was not called');
	return { url: call[0] as string, init: (call[1] as RequestInit) ?? {} };
}

describe('RealApiClient: 인증 헤더', () => {
	it('모든 요청에 Authorization Bearer + Content-Type 헤더 첨부', async () => {
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse([])
		);
		await realApiClient.fetchTags();

		const { init } = lastFetchCall();
		const headers = init.headers as Record<string, string>;
		expect(headers.Authorization).toBe(`Bearer ${FAKE_TOKEN}`);
		expect(headers['Content-Type']).toBe('application/json');
	});

	it('토큰이 null이면 Not authenticated 에러', async () => {
		setRealClientToken(null);
		await expect(realApiClient.fetchTags()).rejects.toThrow('Not authenticated');
	});
});

describe('RealApiClient: tags', () => {
	it('fetchTags → GET /api/tags', async () => {
		const tags = [{ id: '1', name: 'AI', category: 'Tech' }];
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse(tags)
		);

		const result = await realApiClient.fetchTags();
		expect(result).toEqual(tags);
		expect(lastFetchCall().url).toBe('http://localhost:8081/api/tags');
	});

	it('fetchMyTagIds → GET /api/me/tags', async () => {
		const ids = ['t1', 't2'];
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse(ids)
		);

		const result = await realApiClient.fetchMyTagIds();
		expect(result).toEqual(ids);
		expect(lastFetchCall().url).toBe('http://localhost:8081/api/me/tags');
	});

	it('saveMyTags → POST /api/me/tags + tag_ids body', async () => {
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse({ ok: true })
		);

		await realApiClient.saveMyTags(['t1', 't2']);
		const { url, init } = lastFetchCall();
		expect(url).toBe('http://localhost:8081/api/me/tags');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual({ tag_ids: ['t1', 't2'] });
	});

	it('updateMyTags → POST /api/me/tags (saveMyTags와 동일 엔드포인트)', async () => {
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse({ ok: true })
		);

		await realApiClient.updateMyTags(['t3']);
		const { url, init } = lastFetchCall();
		expect(url).toBe('http://localhost:8081/api/me/tags');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual({ tag_ids: ['t3'] });
	});
});

describe('RealApiClient: profile', () => {
	it('fetchProfile → GET /api/me/profile', async () => {
		const profile = { id: 'u1', display_name: 'Alice', onboarding_completed: true };
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse(profile)
		);

		const result = await realApiClient.fetchProfile();
		expect(result).toEqual(profile);
		expect(lastFetchCall().url).toBe('http://localhost:8081/api/me/profile');
	});

	it('updateProfile → PUT /api/me/profile + patch body', async () => {
		const updated = { id: 'u1', display_name: 'Bob', onboarding_completed: true };
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse(updated)
		);

		const result = await realApiClient.updateProfile({ display_name: 'Bob' });
		expect(result).toEqual(updated);

		const { url, init } = lastFetchCall();
		expect(url).toBe('http://localhost:8081/api/me/profile');
		expect(init.method).toBe('PUT');
		expect(JSON.parse(init.body as string)).toEqual({ display_name: 'Bob' });
	});
});

describe('RealApiClient: articles', () => {
	it('fetchArticles 기본 호출 → GET /api/me/articles (no qs)', async () => {
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse([])
		);

		await realApiClient.fetchArticles();
		expect(lastFetchCall().url).toBe('http://localhost:8081/api/me/articles');
	});

	it('fetchArticles + offset/limit/tagId → 정확한 query string', async () => {
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse([])
		);

		await realApiClient.fetchArticles({ offset: 10, limit: 20, tagId: 't1' });
		const { url } = lastFetchCall();
		expect(url).toContain('offset=10');
		expect(url).toContain('limit=20');
		expect(url).toContain('tag_id=t1');
	});

	it('fetchArticleById → GET /api/me/articles/:id', async () => {
		const article = { id: 'a1', title: 'Test', tag_id: null };
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse(article)
		);

		const result = await realApiClient.fetchArticleById('a1');
		expect(result).toEqual(article);
		expect(lastFetchCall().url).toBe('http://localhost:8081/api/me/articles/a1');
	});

	it('fetchArticleById는 404 응답을 null로 변환', async () => {
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse({ error: 'Not found' }, 404)
		);

		const result = await realApiClient.fetchArticleById('missing');
		expect(result).toBeNull();
	});

	it('fetchArticleById는 500 응답을 throw', async () => {
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse({ error: 'Server error' }, 500)
		);

		await expect(realApiClient.fetchArticleById('a1')).rejects.toThrow('Server error');
	});

	it('fetchArticleById는 id를 URL-encode', async () => {
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse({ id: 'a/1' })
		);

		await realApiClient.fetchArticleById('a/1');
		expect(lastFetchCall().url).toBe('http://localhost:8081/api/me/articles/a%2F1');
	});
});

describe('RealApiClient: pipeline', () => {
	it('collectArticles → POST /api/me/collect, returns collected count', async () => {
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse({ collected: 7 })
		);

		const result = await realApiClient.collectArticles();
		expect(result).toBe(7);

		const { url, init } = lastFetchCall();
		expect(url).toBe('http://localhost:8081/api/me/collect');
		expect(init.method).toBe('POST');
	});

	it('summarizeArticles → POST /api/me/summarize, returns summarized count', async () => {
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse({ summarized: 3 })
		);

		const result = await realApiClient.summarizeArticles();
		expect(result).toBe(3);

		const { url, init } = lastFetchCall();
		expect(url).toBe('http://localhost:8081/api/me/summarize');
		expect(init.method).toBe('POST');
	});
});

describe('RealApiClient: 에러 처리', () => {
	it('서버 에러 응답의 error 메시지를 throw', async () => {
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			jsonResponse({ error: 'limit must be >= 1' }, 400)
		);

		await expect(realApiClient.fetchArticles()).rejects.toThrow('limit must be >= 1');
	});

	it('error 필드 없는 에러 응답은 status를 포함한 메시지로 fallback', async () => {
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			emptyResponse(500)
		);

		await expect(realApiClient.fetchTags()).rejects.toThrow('Request failed (500)');
	});

	it('204 No Content 응답을 정상 처리', async () => {
		(globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(
			emptyResponse(204)
		);

		// saveMyTags는 응답 body를 무시하므로 throw 0
		await expect(realApiClient.saveMyTags([])).resolves.toBeUndefined();
	});
});
