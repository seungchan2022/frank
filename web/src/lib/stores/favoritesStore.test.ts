// favoritesStore.svelte.ts 단위 테스트
// Svelte 5 $state 기반 즐겨찾기 store — add/remove/list/guard 검증.

import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { FeedItem } from '$lib/api/types';
import type { Favorite } from '$lib/types/favorite';

// apiClient mock
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

function makeFeedItem(url = 'https://example.com'): FeedItem {
	return {
		title: '테스트 기사',
		url,
		snippet: null,
		source: 'test',
		published_at: null,
		tag_id: null
	};
}

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
		createdAt: new Date().toISOString()
	};
}

beforeEach(async () => {
	vi.resetModules();
	const apiMod = await import('$lib/api');
	apiClient = apiMod.apiClient as FakeApiClient;
	const storeMod = await import('./favoritesStore.svelte');
	favoritesStore = storeMod.favoritesStore;
});

describe('favoritesStore: 초기 상태', () => {
	it('초기 favorites는 빈 배열', () => {
		expect(favoritesStore.favorites).toEqual([]);
	});

	it('초기 loaded는 false', () => {
		expect(favoritesStore.loaded).toBe(false);
	});

	it('초기 isLiked는 false', () => {
		expect(favoritesStore.isLiked('https://example.com')).toBe(false);
	});
});

describe('favoritesStore: loadFavorites', () => {
	it('API 호출 → favorites 채워짐', async () => {
		const fav = makeFavorite();
		vi.mocked(apiClient.listFavorites).mockResolvedValueOnce([fav]);

		await favoritesStore.loadFavorites('user-1');

		expect(favoritesStore.favorites).toHaveLength(1);
		expect(favoritesStore.favorites[0].url).toBe('https://example.com');
		expect(favoritesStore.loaded).toBe(true);
	});

	it('로드 후 loaded=true 설정됨', async () => {
		vi.mocked(apiClient.listFavorites).mockResolvedValue([]);
		await favoritesStore.loadFavorites('user-1');
		expect(favoritesStore.loaded).toBe(true);
	});

	it('빈 즐겨찾기 사용자도 정상 처리 (loaded=true)', async () => {
		vi.mocked(apiClient.listFavorites).mockResolvedValue([]);

		await favoritesStore.loadFavorites('user-1');

		expect(favoritesStore.loaded).toBe(true);
		expect(favoritesStore.favorites).toEqual([]);
	});

	it('userId 전환 → 이전 데이터 초기화 후 재로드', async () => {
		vi.mocked(apiClient.listFavorites)
			.mockResolvedValueOnce([makeFavorite('https://user1.com')])
			.mockResolvedValueOnce([makeFavorite('https://user2.com')]);

		await favoritesStore.loadFavorites('user-1');
		expect(favoritesStore.favorites[0].url).toBe('https://user1.com');

		await favoritesStore.loadFavorites('user-2');
		expect(favoritesStore.favorites[0].url).toBe('https://user2.com');
		expect(favoritesStore.favorites).toHaveLength(1);
	});

	it('API 실패 → error 상태 설정', async () => {
		vi.mocked(apiClient.listFavorites).mockRejectedValueOnce(new Error('Network error'));

		await favoritesStore.loadFavorites('user-1');

		expect(favoritesStore.error).toBeTruthy();
		expect(favoritesStore.favorites).toEqual([]);
	});
});

describe('favoritesStore: addFavorite', () => {
	it('추가 후 favorites에 prepend', async () => {
		const fav = makeFavorite();
		vi.mocked(apiClient.addFavorite).mockResolvedValueOnce(fav);

		const item = makeFeedItem();
		await favoritesStore.addFavorite(item, '요약', '인사이트');

		expect(favoritesStore.favorites).toHaveLength(1);
		expect(favoritesStore.favorites[0].url).toBe('https://example.com');
	});

	it('추가 후 isLiked → true', async () => {
		const fav = makeFavorite();
		vi.mocked(apiClient.addFavorite).mockResolvedValueOnce(fav);

		await favoritesStore.addFavorite(makeFeedItem());

		expect(favoritesStore.isLiked('https://example.com')).toBe(true);
	});

	it('중복 추가 시 API 에러 throw → store 상태 변경 없음', async () => {
		vi.mocked(apiClient.addFavorite).mockResolvedValueOnce(makeFavorite());
		await favoritesStore.addFavorite(makeFeedItem());

		vi.mocked(apiClient.addFavorite).mockRejectedValueOnce(
			Object.assign(new Error('이미 즐겨찾기'), { status: 409 })
		);

		await expect(favoritesStore.addFavorite(makeFeedItem())).rejects.toThrow();
		// 중복 추가 실패 후에도 기존 1개 유지
		expect(favoritesStore.favorites).toHaveLength(1);
	});
});

describe('favoritesStore: removeFavorite', () => {
	it('삭제 후 favorites에서 제거', async () => {
		const fav = makeFavorite();
		vi.mocked(apiClient.addFavorite).mockResolvedValueOnce(fav);
		vi.mocked(apiClient.deleteFavorite).mockResolvedValueOnce(undefined);

		await favoritesStore.addFavorite(makeFeedItem());
		await favoritesStore.removeFavorite('https://example.com');

		expect(favoritesStore.favorites).toHaveLength(0);
		expect(favoritesStore.isLiked('https://example.com')).toBe(false);
	});

	it('삭제 후 isLiked → false', async () => {
		vi.mocked(apiClient.addFavorite).mockResolvedValueOnce(makeFavorite());
		vi.mocked(apiClient.deleteFavorite).mockResolvedValueOnce(undefined);

		await favoritesStore.addFavorite(makeFeedItem());
		expect(favoritesStore.isLiked('https://example.com')).toBe(true);

		await favoritesStore.removeFavorite('https://example.com');
		expect(favoritesStore.isLiked('https://example.com')).toBe(false);
	});
});

describe('favoritesStore: reset', () => {
	it('reset 후 초기 상태로 복귀', async () => {
		vi.mocked(apiClient.listFavorites).mockResolvedValueOnce([makeFavorite()]);
		await favoritesStore.loadFavorites('user-1');

		favoritesStore.reset();

		expect(favoritesStore.favorites).toEqual([]);
		expect(favoritesStore.loaded).toBe(false);
	});
});

describe('favoritesStore: loadFavorites 에러 브랜치', () => {
	it('API 실패 — Error 인스턴스 아닌 경우 fallback 메시지', async () => {
		vi.mocked(apiClient.listFavorites).mockRejectedValueOnce('string-error');

		await favoritesStore.loadFavorites('user-1');

		expect(favoritesStore.error).toBe('즐겨찾기를 불러오지 못했습니다.');
	});
});

describe('favoritesStore: isQuizCompleted', () => {
	it('퀴즈 완료 처리 전 — isQuizCompleted=false', async () => {
		vi.mocked(apiClient.addFavorite).mockResolvedValueOnce(makeFavorite());
		await favoritesStore.addFavorite(makeFeedItem());

		expect(favoritesStore.isQuizCompleted('https://example.com')).toBe(false);
	});

	it('존재하지 않는 URL — isQuizCompleted=false', () => {
		expect(favoritesStore.isQuizCompleted('https://nonexistent.com')).toBe(false);
	});
});
