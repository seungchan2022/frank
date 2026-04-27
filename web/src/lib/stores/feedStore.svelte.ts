/**
 * MVP5 M3: 피드 상태 관리 store.
 * Svelte 5 $state 기반 모듈 수준 상태 — 페이지 remount(뒤로가기 복귀) 시에도 데이터 유지.
 *
 * MVP6 M2: isRefreshing 상태 분리 (stale-while-revalidate).
 * MVP6 M3: 탭별 캐시 (tagCache).
 * MVP12 M2: 'all' 탭 단일 캐시 + 클라이언트 필터링.
 * - 'all' 탭 결과만 서버에서 가져오고, 태그 탭은 클라이언트 필터링
 * - selectTag: API 재호출 없이 activeTagId만 변경
 * - loadMore(): 항상 'all' 탭 기준으로 페이지 append
 */

import { apiClient } from '$lib/api';
import type { FeedItem } from '$lib/types/article';
import type { Tag } from '$lib/types/tag';

/** 페이지당 기사 수 */
const PAGE_SIZE = 20;

/** 탭별 페이지네이션 상태 */
export interface TabState {
	items: FeedItem[];
	nextOffset: number;
	hasMore: boolean;
	status: 'idle' | 'loading' | 'loadingMore' | 'error';
}

let tags = $state<Tag[]>([]);
let myTagIds = $state<string[]>([]);
let loaded = $state(false);
let loading = $state(false);
let isRefreshing = $state(false);
let error = $state<string | null>(null);
/** 현재 선택된 태그 ID. null = 전체 탭 */
let activeTagId = $state<string | null>(null);
/**
 * 탭별 캐시. 키: 태그 ID 또는 'all' (전체 탭)
 * Svelte 5 반응성 규칙: Map 뮤테이션은 반응성을 트리거하지 않으므로 항상 새 Map 할당.
 */
let tagCache = $state(new Map<string, TabState>());

/** 'all' 탭 전체 아이템 */
let allItems = $derived(tagCache.get('all')?.items ?? []);
/** feedItems: activeTagId가 있으면 'all' 기준 클라이언트 필터링 */
let feedItems = $derived(
	activeTagId ? allItems.filter((item) => item.tag_id === activeTagId) : allItems
);
/** hasMore: 항상 'all' 탭 기준 */
let hasMore = $derived(tagCache.get('all')?.hasMore ?? true);
/** isLoadingMore: 'all' 탭 기준 */
let isLoadingMore = $derived(tagCache.get('all')?.status === 'loadingMore');

let loadedForUserId = $state<string | null>(null);

function makeTabState(items: FeedItem[]): TabState {
	return {
		items,
		nextOffset: items.length,
		hasMore: items.length >= PAGE_SIZE,
		status: 'idle'
	};
}

/**
 * 초기 피드 로드. 이미 로드된 경우 no-op.
 * MVP12 M2: 전체 탭만 즉시 로드 (구독 태그 병렬 프리패치 제거).
 */
async function loadFeed(userId?: string): Promise<void> {
	if (loaded && loadedForUserId === (userId ?? null)) return;
	if (loading || isRefreshing) return;

	if (loadedForUserId !== (userId ?? null)) {
		tags = [];
		myTagIds = [];
		loaded = false;
		tagCache = new Map();
		activeTagId = null;
	}

	loading = true;
	error = null;

	try {
		const [items, allTags, tagIds] = await Promise.all([
			apiClient.fetchFeed(undefined, { limit: PAGE_SIZE, offset: 0 }),
			apiClient.fetchTags(),
			apiClient.fetchMyTagIds()
		]);
		tags = allTags;
		myTagIds = tagIds;

		// 전체 탭만 즉시 캐시 (다른 탭은 selectTag 시 lazy fetch)
		tagCache = new Map<string, TabState>([['all', makeTabState(items)]]);

		loaded = true;
		loadedForUserId = userId ?? null;
		activeTagId = null;
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to load feed';
	} finally {
		loading = false;
	}
}

/**
 * 탭 선택.
 * 'all' 탭 결과를 클라이언트에서 필터링 — API 재호출 없이 activeTagId만 변경.
 */
function selectTag(tagId: string | null): void {
	if (loading || isRefreshing) return;
	activeTagId = tagId;
}

/**
 * 다음 페이지 로드 (무한 스크롤용).
 * 항상 'all' 탭 기준 — 태그 필터는 클라이언트 필터링이므로 서버 요청은 'all'만.
 * E-02: hasMore=false 또는 isLoadingMore 중이면 재진입 차단.
 * E-03: capturedOffset으로 refresh race condition 방지.
 */
async function loadMore(): Promise<void> {
	if (isRefreshing) return;

	const key = 'all';
	const current = tagCache.get(key);

	// E-02: 재진입 가드
	if (!current || !current.hasMore || current.status === 'loadingMore') return;

	const capturedOffset = current.nextOffset;

	tagCache = new Map([...tagCache, [key, { ...current, status: 'loadingMore' as const }]]);

	try {
		const items = await apiClient.fetchFeed(undefined, { limit: PAGE_SIZE, offset: capturedOffset });

		const latest = tagCache.get(key);
		if (!latest) return;

		// E-03: refresh로 인해 nextOffset이 리셋된 경우 stale 응답 버림
		if (capturedOffset !== latest.nextOffset) return;

		tagCache = new Map([
			...tagCache,
			[key, {
				items: [...latest.items, ...items],
				nextOffset: capturedOffset + items.length,
				hasMore: items.length >= PAGE_SIZE,
				status: 'idle' as const
			}]
		]);
	} catch (e) {
		const latest = tagCache.get(key);
		if (latest) {
			tagCache = new Map([...tagCache, [key, { ...latest, status: 'error' as const }]]);
		}
		error = e instanceof Error ? e.message : 'Failed to load more';
	}
}

/**
 * 새 뉴스 가져오기 — 강제 재요청. 현재 탭 pagination 리셋 후 첫 페이지 재요청.
 * loadingMore 진행 중에도 차단 — loadMore의 stale 응답이 refresh 결과를 덮어쓰는 역방향 race 방지.
 */
async function refresh(): Promise<boolean> {
	if (loading || isRefreshing) return false;
	// loadMore 진행 중이면 refresh 차단 (역방향 race condition)
	if (isLoadingMore) return false;

	isRefreshing = true;
	error = null;

	try {
		const items = await apiClient.fetchFeed(
			undefined,
			{ noCache: true, limit: PAGE_SIZE, offset: 0 }
		);
		// 'all' 탭 pagination 리셋 (태그 탭은 클라이언트 필터링)
		tagCache = new Map([...tagCache, ['all', makeTabState(items)]]);
		return true;
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to refresh feed';
		return false;
	} finally {
		isRefreshing = false;
	}
}

/**
 * 상태 완전 초기화 (로그아웃 등 세션 전환 시).
 */
function reset(): void {
	tags = [];
	myTagIds = [];
	loaded = false;
	loadedForUserId = null;
	loading = false;
	isRefreshing = false;
	error = null;
	tagCache = new Map();
	activeTagId = null;
}

export const feedStore = {
	get feedItems() {
		return feedItems;
	},
	get tags() {
		return tags;
	},
	get myTagIds() {
		return myTagIds;
	},
	get loaded() {
		return loaded;
	},
	get loading() {
		return loading;
	},
	get isRefreshing() {
		return isRefreshing;
	},
	get error() {
		return error;
	},
	get activeTagId() {
		return activeTagId;
	},
	get hasMore() {
		return hasMore;
	},
	get isLoadingMore() {
		return isLoadingMore;
	},
	loadFeed,
	selectTag,
	loadMore,
	refresh,
	reset
};
