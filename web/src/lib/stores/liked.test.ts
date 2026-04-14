/**
 * MVP7 M2: likedStore 단위 테스트.
 * Svelte 5 $state 기반 — Set 참조 교체 패턴 검증.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { createLikedStore } from './liked.svelte';

// POST /api/articles/like 프록시 mock
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));

describe('likedStore', () => {
	let store: ReturnType<typeof createLikedStore>;

	const mockFetch = (response: unknown, ok = true) => {
		global.fetch = vi.fn().mockResolvedValueOnce({
			ok,
			json: () => Promise.resolve(response)
		} as Response);
	};

	beforeEach(() => {
		store = createLikedStore();
		vi.clearAllMocks();
	});

	it('초기 상태: likedUrls는 빈 Set', () => {
		expect(store.likedUrls.size).toBe(0);
	});

	it('likeArticle 성공 시 likedUrls에 url 추가', async () => {
		const url = 'https://example.com/article1';
		mockFetch({ keywords: ['iOS', 'Swift'], total_likes: 1 });

		await store.likeArticle({ url, title: 'iOS 기사', snippet: 'Swift 내용' });

		expect(store.likedUrls.has(url)).toBe(true);
	});

	it('likeArticle: tag_id를 요청 바디에 포함', async () => {
		const url = 'https://example.com/article-tag';
		const tag_id = '11111111-1111-1111-1111-111111111111';
		mockFetch({ keywords: ['iOS'], total_likes: 1 });

		await store.likeArticle({ url, title: '기사', snippet: null, tag_id });

		const call = (global.fetch as ReturnType<typeof vi.fn>).mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.tag_id).toBe(tag_id);
	});

	it('isLiked: likedUrls에 있으면 true 반환', async () => {
		const url = 'https://example.com/article2';
		mockFetch({ keywords: ['Swift'], total_likes: 1 });

		await store.likeArticle({ url, title: '기사', snippet: null });

		expect(store.isLiked(url)).toBe(true);
		expect(store.isLiked('https://other.com')).toBe(false);
	});

	it('이미 liked url은 API 호출 없이 반환', async () => {
		const url = 'https://example.com/dup';
		mockFetch({ keywords: ['iOS'], total_likes: 1 });

		// 첫 번째 호출
		await store.likeArticle({ url, title: '기사', snippet: null });
		// 두 번째 호출 (중복)
		await store.likeArticle({ url, title: '기사', snippet: null });

		// fetch는 1번만 호출됨
		expect(global.fetch).toHaveBeenCalledTimes(1);
	});

	it('API 실패해도 likedUrls에 즉시 추가됨 (fire-and-forget)', async () => {
		const url = 'https://example.com/fail';
		global.fetch = vi.fn().mockRejectedValueOnce(new Error('Network error'));

		store.likeArticle({ url, title: '실패 기사', snippet: null });

		// fire-and-forget: API 실패와 무관하게 즉시 UI 반영
		expect(store.likedUrls.has(url)).toBe(true);
	});

	it('likedUrls는 Svelte 5 참조 교체 패턴 — Set 뮤테이션 금지', async () => {
		const url = 'https://example.com/ref-test';
		mockFetch({ keywords: ['iOS'], total_likes: 1 });

		const before = store.likedUrls;
		await store.likeArticle({ url, title: '기사', snippet: null });
		const after = store.likedUrls;

		// 새 Set 인스턴스여야 함 (참조 교체)
		expect(after).not.toBe(before);
	});
});
