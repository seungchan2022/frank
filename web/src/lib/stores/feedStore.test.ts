// feedStore.svelte.ts 단위 테스트
// MVP12 M2: TabState + loadMore + lazy selectTag 전략 반영 (ST4 재작성)

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

const PAGE_SIZE = 20;

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

function makeFeedItems(count: number, prefix = 'https://example.com/news/'): FeedItem[] {
	return Array.from({ length: count }, (_, i) => makeFeedItem(`${prefix}${i}`));
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

	it('초기 hasMore는 true', () => {
		expect(feedStore.hasMore).toBe(true);
	});

	it('초기 isLoadingMore는 false', () => {
		expect(feedStore.isLoadingMore).toBe(false);
	});
});

describe('feedStore: loadFeed', () => {
	it('API 호출 → feedItems 채워짐', async () => {
		const items = makeFeedItems(5);
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(items);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);

		await feedStore.loadFeed('user-1');

		expect(feedStore.feedItems).toHaveLength(5);
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

	it('ST4: loadFeed는 전체 탭만 즉시 로드 — 구독 태그 병렬 프리패치 없음', async () => {
		vi.mocked(apiClient.fetchFeed).mockClear();
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(makeFeedItems(3));
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([makeTag('tag-1', 'AI')]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce(['tag-1']);

		await feedStore.loadFeed('user-1');

		// 전체 피드만 1회 호출 (tag-1 프리패치 없음)
		expect(vi.mocked(apiClient.fetchFeed)).toHaveBeenCalledTimes(1);
		expect(vi.mocked(apiClient.fetchFeed)).toHaveBeenCalledWith(undefined, expect.objectContaining({ limit: PAGE_SIZE, offset: 0 }));
	});

	it('PAGE_SIZE 미만 응답 시 hasMore=false', async () => {
		const items = makeFeedItems(5); // 20 미만
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(items);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);

		await feedStore.loadFeed('user-1');

		expect(feedStore.hasMore).toBe(false);
	});

	it('PAGE_SIZE 응답 시 hasMore=true', async () => {
		const items = makeFeedItems(PAGE_SIZE);
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(items);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);

		await feedStore.loadFeed('user-1');

		expect(feedStore.hasMore).toBe(true);
	});
});

describe('feedStore: loadMore (T-03)', () => {
	it('T-03a: loadMore 정상 — 다음 페이지 items 누적', async () => {
		// 초기 로드
		const firstPage = makeFeedItems(PAGE_SIZE, 'https://example.com/a/');
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(firstPage);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);
		await feedStore.loadFeed('user-1');

		// 두 번째 페이지
		const secondPage = makeFeedItems(5, 'https://example.com/b/');
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(secondPage);
		await feedStore.loadMore();

		expect(feedStore.feedItems).toHaveLength(PAGE_SIZE + 5);
		expect(feedStore.hasMore).toBe(false);
	});

	it('T-03b: hasMore=false 시 loadMore는 no-op', async () => {
		const items = makeFeedItems(5); // PAGE_SIZE 미만 → hasMore=false
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(items);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);
		await feedStore.loadFeed('user-1');

		vi.mocked(apiClient.fetchFeed).mockClear();
		await feedStore.loadMore();

		expect(vi.mocked(apiClient.fetchFeed)).not.toHaveBeenCalled();
	});

	it('T-03c: isLoadingMore 중 loadMore 재진입 차단', async () => {
		const firstPage = makeFeedItems(PAGE_SIZE);
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(firstPage);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);
		await feedStore.loadFeed('user-1');

		vi.mocked(apiClient.fetchFeed).mockClear();
		let resolveMore!: (v: FeedItem[]) => void;
		const morePromise = new Promise<FeedItem[]>((res) => { resolveMore = res; });
		vi.mocked(apiClient.fetchFeed).mockReturnValueOnce(morePromise);

		const firstMore = feedStore.loadMore();
		// 이미 로딩 중 — 재진입 차단
		await feedStore.loadMore();

		expect(vi.mocked(apiClient.fetchFeed).mock.calls.length).toBe(1);

		resolveMore([]);
		await firstMore;
	});
});

describe('feedStore: selectTag — 캐시 직접 참조 (T-04, MVP13 M2)', () => {
	it('T-04a: selectTag → API 재호출 없이 activeTagId만 변경, 캐시에서 직접 조회', async () => {
		const items: FeedItem[] = [
			{ ...makeFeedItem('https://a.com/1'), tag_id: 'tag-1' },
			{ ...makeFeedItem('https://a.com/2'), tag_id: 'tag-1' },
			{ ...makeFeedItem('https://a.com/3'), tag_id: 'tag-2' }
		];
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(items);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([makeTag('tag-1', 'AI')]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce(['tag-1']);
		await feedStore.loadFeed('user-1');

		vi.mocked(apiClient.fetchFeed).mockClear();
		feedStore.selectTag('tag-1');

		// MVP13 M2: API 재호출 없음 (loadFeed 시 분리 저장)
		expect(vi.mocked(apiClient.fetchFeed)).not.toHaveBeenCalled();
		expect(feedStore.activeTagId).toBe('tag-1');
		// 캐시에서 직접 조회 — tag-1 기사 2개
		expect(feedStore.feedItems).toHaveLength(2);
	});

	it('T-04b: 같은 태그 재선택 → API 없음, activeTagId 유지', async () => {
		const items: FeedItem[] = [
			{ ...makeFeedItem('https://a.com/1'), tag_id: 'tag-1' },
			{ ...makeFeedItem('https://a.com/2'), tag_id: 'tag-2' }
		];
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(items);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([makeTag('tag-1', 'AI')]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce(['tag-1']);
		await feedStore.loadFeed('user-1');

		feedStore.selectTag('tag-1');
		vi.mocked(apiClient.fetchFeed).mockClear();
		feedStore.selectTag('tag-1');

		expect(vi.mocked(apiClient.fetchFeed)).not.toHaveBeenCalled();
	});

	it('T-04c: 전체 탭 복귀(null) → API 없음, 전체 items 반환', async () => {
		const items: FeedItem[] = [
			{ ...makeFeedItem('https://a.com/1'), tag_id: 'tag-1' },
			{ ...makeFeedItem('https://a.com/2'), tag_id: 'tag-2' }
		];
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(items);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([makeTag('tag-1', 'AI')]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce(['tag-1']);
		await feedStore.loadFeed('user-1');

		feedStore.selectTag('tag-1');
		vi.mocked(apiClient.fetchFeed).mockClear();
		feedStore.selectTag(null);

		expect(vi.mocked(apiClient.fetchFeed)).not.toHaveBeenCalled();
		expect(feedStore.activeTagId).toBeNull();
		expect(feedStore.feedItems).toHaveLength(2); // 전체 반환
	});
});

describe('feedStore: refresh', () => {
	it('refresh → 현재 탭 pagination 리셋 후 첫 페이지 재요청', async () => {
		const firstPage = makeFeedItems(PAGE_SIZE);
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(firstPage);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);
		await feedStore.loadFeed('user-1');

		// loadMore 한 번
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(makeFeedItems(5, 'https://b.com/'));
		await feedStore.loadMore();
		expect(feedStore.feedItems).toHaveLength(PAGE_SIZE + 5);

		// refresh → 첫 페이지로 리셋
		const refreshedItems = makeFeedItems(PAGE_SIZE, 'https://new.com/');
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(refreshedItems);
		await feedStore.refresh();

		expect(feedStore.feedItems).toHaveLength(PAGE_SIZE);
		expect(feedStore.feedItems[0].url).toBe('https://new.com/0');
	});

	it('refresh 실패 → error 설정', async () => {
		vi.mocked(apiClient.fetchFeed).mockRejectedValueOnce(new Error('Refresh failed'));

		await feedStore.refresh();

		expect(feedStore.error).toBeTruthy();
	});
});

describe('feedStore: isRefreshing 상태 분리 (stale-while-revalidate)', () => {
	it('refresh 중에는 isRefreshing이 true, loading은 false', async () => {
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce([makeFeedItem()]);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);
		await feedStore.loadFeed('user-1');

		let resolveRefresh!: (value: FeedItem[]) => void;
		const refreshPromise = new Promise<FeedItem[]>((res) => { resolveRefresh = res; });
		vi.mocked(apiClient.fetchFeed).mockReturnValueOnce(refreshPromise);

		const refreshing = feedStore.refresh();

		expect(feedStore.isRefreshing).toBe(true);
		expect(feedStore.loading).toBe(false);

		resolveRefresh([makeFeedItem('https://example.com/news/new')]);
		await refreshing;
	});

	it('refresh 중에도 이전 feedItems 유지됨', async () => {
		const oldItem = makeFeedItem('https://example.com/news/old');
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce([oldItem]);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);
		await feedStore.loadFeed('user-1');

		let resolveRefresh!: (value: FeedItem[]) => void;
		const refreshPromise = new Promise<FeedItem[]>((res) => { resolveRefresh = res; });
		vi.mocked(apiClient.fetchFeed).mockReturnValueOnce(refreshPromise);

		const refreshing = feedStore.refresh();

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

		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce([makeFeedItem('https://example.com/news/new')]);
		await feedStore.refresh();

		expect(feedStore.isRefreshing).toBe(false);
	});

	it('loadFeed 진행 중 refresh 호출 시 no-op', async () => {
		let resolveLoad!: (value: FeedItem[]) => void;
		const loadPromise = new Promise<FeedItem[]>((res) => { resolveLoad = res; });
		vi.mocked(apiClient.fetchFeed).mockReturnValueOnce(loadPromise);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);

		const loading = feedStore.loadFeed('user-1');

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

		vi.mocked(apiClient.fetchFeed).mockClear();

		let resolveRefresh!: (value: FeedItem[]) => void;
		const refreshPromise = new Promise<FeedItem[]>((res) => { resolveRefresh = res; });
		vi.mocked(apiClient.fetchFeed).mockReturnValueOnce(refreshPromise);

		const refreshing = feedStore.refresh();

		await feedStore.refresh();
		expect(vi.mocked(apiClient.fetchFeed).mock.calls.length).toBe(1);

		resolveRefresh([makeFeedItem('https://example.com/news/new')]);
		await refreshing;
		expect(feedStore.isRefreshing).toBe(false);
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
		expect(feedStore.hasMore).toBe(true);
	});
});

// selectTag는 API를 호출하지 않으므로 (클라이언트 필터링) 에러 케이스 없음

describe('feedStore: loadMore 에러 처리 (lines 161-177)', () => {
	it('loadMore API 실패 → error 설정, status=error로 복구', async () => {
		const firstPage = makeFeedItems(PAGE_SIZE);
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(firstPage);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);
		await feedStore.loadFeed('user-1');

		vi.mocked(apiClient.fetchFeed).mockRejectedValueOnce(new Error('Load more failed'));
		await feedStore.loadMore();

		expect(feedStore.error).toBeTruthy();
		// hasMore는 false가 됨 (status=error 탭에서 재로딩 불가)
		expect(feedStore.isLoadingMore).toBe(false);
	});

	it('loadMore 에러 문자열 분기 — Error 인스턴스 아닌 경우', async () => {
		const firstPage = makeFeedItems(PAGE_SIZE);
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(firstPage);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce([]);
		await feedStore.loadFeed('user-1');

		vi.mocked(apiClient.fetchFeed).mockRejectedValueOnce('string-error');
		await feedStore.loadMore();

		expect(feedStore.error).toBe('Failed to load more');
	});
});

describe('feedStore: MVP13 M2 — 초기 로드 시 태그별 분리 저장', () => {
	it('loadFeed 후 tag_id별 캐시 분리 저장 — selectTag 시 API 없음', async () => {
		const items: FeedItem[] = [
			{ ...makeFeedItem('https://a.com/1'), tag_id: 'tag-1' },
			{ ...makeFeedItem('https://a.com/2'), tag_id: 'tag-1' },
			{ ...makeFeedItem('https://a.com/3'), tag_id: 'tag-2' },
			{ ...makeFeedItem('https://a.com/4'), tag_id: null } // 태그 없음
		];
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(items);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([
			makeTag('tag-1', 'AI'),
			makeTag('tag-2', '경제')
		]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce(['tag-1', 'tag-2']);
		await feedStore.loadFeed('user-1');

		// 전체 탭: 4개 모두
		expect(feedStore.feedItems).toHaveLength(4);

		vi.mocked(apiClient.fetchFeed).mockClear();

		// tag-1 탭으로 전환 — API 없음, 분리 캐시에서 직접 조회
		feedStore.selectTag('tag-1');
		expect(vi.mocked(apiClient.fetchFeed)).not.toHaveBeenCalled();
		expect(feedStore.feedItems).toHaveLength(2); // tag-1 기사 2개

		// tag-2 탭으로 전환
		feedStore.selectTag('tag-2');
		expect(vi.mocked(apiClient.fetchFeed)).not.toHaveBeenCalled();
		expect(feedStore.feedItems).toHaveLength(1); // tag-2 기사 1개
	});

	it('태그 탭의 hasMore는 false (태그 탭 loadMore 없음 — F-08)', async () => {
		const items: FeedItem[] = Array.from({ length: 20 }, (_, i) => ({
			...makeFeedItem(`https://a.com/${i}`),
			tag_id: 'tag-1'
		}));
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(items);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([makeTag('tag-1', 'AI')]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce(['tag-1']);
		await feedStore.loadFeed('user-1');

		feedStore.selectTag('tag-1');

		// 태그 탭 hasMore=false (loadMore 없음)
		expect(feedStore.hasMore).toBe(false);
	});

	it('refresh 후 태그별 캐시 재분리 저장', async () => {
		const initialItems: FeedItem[] = [
			{ ...makeFeedItem('https://a.com/1'), tag_id: 'tag-1' }
		];
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(initialItems);
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([makeTag('tag-1', 'AI')]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce(['tag-1']);
		await feedStore.loadFeed('user-1');

		// refresh 후 새 아이템 (tag-1 + tag-2)
		const refreshedItems: FeedItem[] = [
			{ ...makeFeedItem('https://b.com/1'), tag_id: 'tag-1' },
			{ ...makeFeedItem('https://b.com/2'), tag_id: 'tag-2' }
		];
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(refreshedItems);
		await feedStore.refresh();

		// 전체 탭
		expect(feedStore.feedItems).toHaveLength(2);

		vi.mocked(apiClient.fetchFeed).mockClear();

		// tag-1 탭 — refresh 후 재분리 저장 확인
		feedStore.selectTag('tag-1');
		expect(vi.mocked(apiClient.fetchFeed)).not.toHaveBeenCalled();
		expect(feedStore.feedItems).toHaveLength(1);
		expect(feedStore.feedItems[0].url).toBe('https://b.com/1');
	});
});

// MARK: - ST-07 BUG-008: 탭 전환 깜빡임 제거 테스트

describe('feedStore: BUG-008 — 탭 전환 캐시 미스 깜빡임 방지', () => {
	it('캐시 미스 selectTag: isTagLoading이 true인 상태로 activeTagId가 바뀌어야 함 (EmptyState 미노출)', async () => {
		// loadFeed 없이 selectTag 직접 호출 — 캐시 미스 상황
		let fetchResolveFn!: (value: FeedItem[]) => void;
		const fetchPromise = new Promise<FeedItem[]>((resolve) => {
			fetchResolveFn = resolve;
		});
		vi.mocked(apiClient.fetchFeed).mockReturnValueOnce(fetchPromise);

		// selectTag 호출 (비동기, 아직 완료 안 됨)
		const selectPromise = feedStore.selectTag('tag-unknown');

		// fetch 완료 전: isTagLoading=true여야 함 (EmptyState 미표시 보장)
		expect(feedStore.activeTagId).toBe('tag-unknown');
		expect(feedStore.isTagLoading).toBe(true);
		expect(feedStore.feedItems).toHaveLength(0); // 로딩 중은 items=[] 허용

		// fetch 완료
		fetchResolveFn([makeFeedItem('https://example.com/news/loaded')]);
		await selectPromise;

		expect(feedStore.isTagLoading).toBe(false);
		expect(feedStore.feedItems).toHaveLength(1);
	});

	it('캐시 미스 selectTag 에러: 에러 캐시 저장 없이 키 제거 → 재시도 가능', async () => {
		vi.mocked(apiClient.fetchFeed).mockRejectedValueOnce(new Error('network error'));
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([makeTag('tag-1', 'AI')]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce(['tag-1']);
		await feedStore.loadFeed('user-1');

		const originalTagId = feedStore.activeTagId;

		// 캐시에 없는 태그로 selectTag — 에러 발생
		vi.mocked(apiClient.fetchFeed).mockRejectedValueOnce(new Error('tag fetch error'));
		await feedStore.selectTag('tag-nonexistent');

		// 에러 시: activeTagId가 원래 탭으로 롤백되어야 함
		expect(feedStore.activeTagId).toBe(originalTagId);

		// 에러 시: 캐시에 status=error인 항목이 남으면 안 됨 (재시도 불가 차단 방지)
		// 다음 selectTag 재호출 시 API를 다시 호출해야 함
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce([makeFeedItem('https://example.com/retry')]);
		await feedStore.selectTag('tag-nonexistent');

		expect(feedStore.activeTagId).toBe('tag-nonexistent');
		expect(feedStore.feedItems).toHaveLength(1);
		expect(feedStore.feedItems[0].url).toBe('https://example.com/retry');
	});
});

describe('feedStore: G-03 전체 탭 재방문 캐시 히트', () => {
	it('G-03: selectTag(null) 복귀 시 API 없음, 전체 items 반환', async () => {
		vi.mocked(apiClient.fetchFeed).mockResolvedValueOnce(makeFeedItems(3));
		vi.mocked(apiClient.fetchTags).mockResolvedValueOnce([makeTag('tag-1', 'AI')]);
		vi.mocked(apiClient.fetchMyTagIds).mockResolvedValueOnce(['tag-1']);
		await feedStore.loadFeed('user-1');

		feedStore.selectTag('tag-1');
		vi.mocked(apiClient.fetchFeed).mockClear();
		feedStore.selectTag(null);

		expect(vi.mocked(apiClient.fetchFeed)).not.toHaveBeenCalled();
		expect(feedStore.feedItems).toHaveLength(3);
	});
});
