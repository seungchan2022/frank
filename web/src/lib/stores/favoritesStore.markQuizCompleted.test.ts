// MVP8 M3: favoritesStore.markQuizCompleted 단위 테스트

import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { Favorite } from '$lib/types/favorite';

vi.mock('$lib/api', () => ({
	apiClient: {
		addFavorite: vi.fn(),
		deleteFavorite: vi.fn(),
		listFavorites: vi.fn()
	}
}));

type FakeApiClient = {
	addFavorite: ReturnType<typeof vi.fn>;
	deleteFavorite: ReturnType<typeof vi.fn>;
	listFavorites: ReturnType<typeof vi.fn>;
};

let favoritesStore: typeof import('./favoritesStore.svelte').favoritesStore;
let apiClient: FakeApiClient;

function makeFavorite(url = 'https://example.com'): Favorite {
	return {
		id: crypto.randomUUID(),
		userId: 'user-1',
		title: '테스트 기사',
		url,
		snippet: null,
		source: 'test',
		publishedAt: null,
		tagId: null,
		summary: null,
		insight: null,
		likedAt: new Date().toISOString(),
		createdAt: new Date().toISOString(),
		quizCompleted: false
	};
}

beforeEach(async () => {
	vi.resetModules();
	const apiMod = await import('$lib/api');
	apiClient = apiMod.apiClient as FakeApiClient;
	const storeMod = await import('./favoritesStore.svelte');
	favoritesStore = storeMod.favoritesStore;
});

describe('favoritesStore: markQuizCompleted', () => {
	it('markQuizCompleted 호출 후 해당 URL의 quizCompleted가 true로 변경됨', async () => {
		const fav = makeFavorite('https://article.com');
		vi.mocked(apiClient.listFavorites).mockResolvedValueOnce([fav]);
		await favoritesStore.loadFavorites('user-1');

		favoritesStore.markQuizCompleted('https://article.com');

		const updated = favoritesStore.favorites.find((f) => f.url === 'https://article.com');
		expect(updated?.quizCompleted).toBe(true);
	});

	it('다른 URL의 quizCompleted는 변경되지 않음', async () => {
		const fav1 = makeFavorite('https://article1.com');
		const fav2 = makeFavorite('https://article2.com');
		vi.mocked(apiClient.listFavorites).mockResolvedValueOnce([fav1, fav2]);
		await favoritesStore.loadFavorites('user-1');

		favoritesStore.markQuizCompleted('https://article1.com');

		const other = favoritesStore.favorites.find((f) => f.url === 'https://article2.com');
		expect(other?.quizCompleted).toBe(false);
	});

	it('존재하지 않는 URL에 markQuizCompleted 호출 시 에러 없이 무시', async () => {
		vi.mocked(apiClient.listFavorites).mockResolvedValueOnce([makeFavorite()]);
		await favoritesStore.loadFavorites('user-1');

		expect(() => favoritesStore.markQuizCompleted('https://nonexistent.com')).not.toThrow();
		expect(favoritesStore.favorites).toHaveLength(1);
	});
});
