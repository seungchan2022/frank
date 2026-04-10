// feedStore.svelte.ts 단위 테스트
// Svelte 5 $state 기반 피드 store — 초기 로드/중복 방지/refresh/reset 검증.
// favoritesStore.test.ts와 동일하게 vi.resetModules() + 동적 import 패턴 사용.

import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { FeedItem } from '$lib/types/article';
import type { Tag } from '$lib/types/tag';

// apiClient mock
vi.mock('$lib/api', () => ({
	apiClient: {
		fetchFeed: vi.fn(),
		fetchTags: vi.fn(),
		fetchMyTagIds: vi.fn()
	}
}));

type FakeApiClient = {
	fetchFeed: ReturnType<typeof vi.fn>;
	fetchTags: ReturnType<typeof vi.fn>;
	fetchMyTagIds: ReturnType<typeof vi.fn>;
};

let feedStore: typeof import('./feedStore.svelte').feedStore;
let apiClient: FakeApiClient;

function makeFeedItem(url = 'https://example.com/news/1'): FeedItem {
	return {
		title: '테스트 기사',
		url,
		snippet: '테스트 스니펫',
		source: 'TechCrunch',
		published_at: '2026-04-09T10:00:00Z',
		tag_id: null
	};
}

function makeTag(id = 'tag-1', name = 'AI'): Tag {
	return { id, name };
}

beforeEach(async () => {
	vi.resetModules();
	const apiMod = await import('$lib/api');
	apiClient = apiMod.apiClient as FakeApiClient;
	const storeMod = await import('./feedStore.svelte');
	feedStore = storeMod.feedStore;
});

describe('feedStore: 초기 상태', () => {
	it('초기 feedItems는 빈 배열', () => {
		expect(feedStore.feedItems).toEqual([]);
	});

	it('초기 loaded는 false', () => {
		expect(feedStore.loaded).toBe(false);
	});

	it('초기 loading은 false', () => {
		expect(feedStore.loading).toBe(false);
	});

	it('초기 error는 null', () => {
		expect(feedStore.error).toBeNull();
	});
});

describe('feedStore: loadFeed', () => {
	it('API 호출 → feedItems 채워짐', async () => {
		const item = makeFeedItem();
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce([item]);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);

		await feedStore.loadFeed('user-1');

		expect(feedStore.feedItems).toHaveLength(1);
		expect(feedStore.feedItems[0].url).toBe('https://example.com/news/1');
		expect(feedStore.loaded).toBe(true);
	});

	it('tags + myTagIds도 채워짐', async () => {
		const tag = makeTag();
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([tag]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce(['tag-1']);

		await feedStore.loadFeed('user-1');

		expect(feedStore.tags).toHaveLength(1);
		expect(feedStore.myTagIds).toContain('tag-1');
	});

	it('API 실패 → error 설정, feedItems 빈 배열', async () => {
		vi.mocked(apiClient.fetchFeed).mockRejectedValueOnce(new Error('Network error'));
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);

		await feedStore.loadFeed('user-1');

		expect(feedStore.error).toBeTruthy();
		expect(feedStore.feedItems).toEqual([]);
	});
});

describe('feedStore: refresh', () => {
	it('refresh → 새 feedItems로 갱신', async () => {
		const newItem = makeFeedItem('https://example.com/news/new');
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce([newItem]);

		await feedStore.refresh();

		expect(feedStore.feedItems).toHaveLength(1);
		expect(feedStore.feedItems[0].url).toBe('https://example.com/news/new');
	});

	it('refresh 실패 → error 설정', async () => {
		vi.mocked(apiClient.fetchFeed).mockRejectedValueOnce(new Error('Refresh failed'));

		await feedStore.refresh();

		expect(feedStore.error).toBeTruthy();
	});
});

describe('feedStore: reset', () => {
	it('reset 후 초기 상태로 복귀', async () => {
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce([makeFeedItem()]);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);

		await feedStore.loadFeed('user-1');

		feedStore.reset();

		expect(feedStore.feedItems).toEqual([]);
		expect(feedStore.loaded).toBe(false);
		expect(feedStore.error).toBeNull();
	});
});

describe('feedStore: isRefreshing 상태 분리 (stale-while-revalidate)', () => {
	it('refresh 중에는 isRefreshing이 true, loading은 false', async () => {
		// 초기 로드 완료
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce([makeFeedItem()]);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);
		await feedStore.loadFeed('user-1');

		// refresh 중 상태를 캡처하기 위해 Promise로 제어
		let resolveRefresh!: (value: typeof import('$lib/types/article').FeedItem[]) => void;
		const refreshPromise = new Promise<typeof import('$lib/types/article').FeedItem[]>((res) => {
			resolveRefresh = res;
		});
		vi.mocked(apiClient.fetchFeed).mockReturnValueOnce(refreshPromise);

		const refreshing = feedStore.refresh();

		// 아직 완료되지 않은 상태에서 검증
		expect(feedStore.isRefreshing).toBe(true);
		expect(feedStore.loading).toBe(false);

		// 완료 처리
		resolveRefresh([makeFeedItem('https://example.com/news/new')]);
		await refreshing;
	});

	it('refresh 중에도 이전 feedItems 유지됨', async () => {
		const oldItem = makeFeedItem('https://example.com/news/old');
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce([oldItem]);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);
		await feedStore.loadFeed('user-1');

		let resolveRefresh!: (value: typeof import('$lib/types/article').FeedItem[]) => void;
		const refreshPromise = new Promise<typeof import('$lib/types/article').FeedItem[]>((res) => {
			resolveRefresh = res;
		});
		vi.mocked(apiClient.fetchFeed).mockReturnValueOnce(refreshPromise);

		const refreshing = feedStore.refresh();

		// refresh 중에 이전 아이템 유지
		expect(feedStore.feedItems).toHaveLength(1);
		expect(feedStore.feedItems[0].url).toBe('https://example.com/news/old');

		resolveRefresh([makeFeedItem('https://example.com/news/new')]);
		await refreshing;
	});

	it('refresh 완료 후 isRefreshing false로 복귀', async () => {
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce([makeFeedItem()]);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);
		await feedStore.loadFeed('user-1');

		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce([
			makeFeedItem('https://example.com/news/new')
		]);
		await feedStore.refresh();

		expect(feedStore.isRefreshing).toBe(false);
	});

	it('loadFeed 진행 중 refresh 호출 시 no-op', async () => {
		let resolveLoad!: (value: typeof import('$lib/types/article').FeedItem[]) => void;
		const loadPromise = new Promise<typeof import('$lib/types/article').FeedItem[]>((res) => {
			resolveLoad = res;
		});
		vi.mocked(apiClient.fetchFeed).mockReturnValueOnce(loadPromise);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);

		const loading = feedStore.loadFeed('user-1');

		// loadFeed 진행 중 refresh 호출 → no-op
		await feedStore.refresh();
		expect(feedStore.isRefreshing).toBe(false);

		resolveLoad([makeFeedItem()]);
		await loading;
	});

	it('refresh 진행 중 재호출 시 no-op', async () => {
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce([makeFeedItem()]);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);
		await feedStore.loadFeed('user-1');

		// fetchFeed mock 초기화 후 카운트 시작
		vi.mocked(apiClient.fetchFeed).mockClear();

		let resolveRefresh!: (value: typeof import('$lib/types/article').FeedItem[]) => void;
		const refreshPromise = new Promise<typeof import('$lib/types/article').FeedItem[]>((res) => {
			resolveRefresh = res;
		});
		vi.mocked(apiClient.fetchFeed).mockReturnValueOnce(refreshPromise);

		const refreshing = feedStore.refresh();

		// 진행 중 재호출 → no-op (fetchFeed가 두 번 호출되지 않음)
		await feedStore.refresh();
		// 두 번째 refresh는 no-op이므로 fetchFeed는 1번만 호출됨
		expect(vi.mocked(apiClient.fetchFeed).mock.calls.length).toBe(1);

		resolveRefresh([makeFeedItem('https://example.com/news/new')]);
		await refreshing;
		expect(feedStore.isRefreshing).toBe(false);
	});
});
