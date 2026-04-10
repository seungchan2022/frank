/**
 * MVP5 M3: 피드 상태 관리 store.
 * Svelte 5 $state 기반 모듈 수준 상태 — 페이지 remount(뒤로가기 복귀) 시에도 데이터 유지.
 * SvelteKit 라우터가 +page.svelte를 재마운트해도 피드를 재요청하지 않는다.
 *
 * MVP6 M2: isRefreshing 상태 분리 (stale-while-revalidate).
 * - loading: 초기 로드 중 (feedItems 없음)
 * - isRefreshing: 갱신 중 (feedItems 유지, 상단 progress bar 표시)
 *
 * MVP6 M3: 탭별 캐시 (tagCache) + 초기 프리패치.
 * - loadFeed 시 구독 태그 전체를 병렬 프리패치 → 탭 전환은 항상 즉시 표시
 * - pull-to-refresh만 현재 탭 캐시 무효화 + 재요청
 */

import { apiClient } from '$lib/api';
import type { FeedItem } from '$lib/types/article';
import type { Tag } from '$lib/types/tag';

let feedItems = $state<FeedItem[]>([]);
let tags = $state<Tag[]>([]);
let myTagIds = $state<string[]>([]);
let loaded = $state(false);
let loading = $state(false);
let isRefreshing = $state(false);
let error = $state<string | null>(null);
/** 현재 선택된 태그 ID. null = 전체 탭 */
let activeTagId = $state<string | null>(null);
/** 탭별 캐시. 키: 태그 ID 또는 'all' (전체 탭) */
let tagCache = $state(new Map<string, FeedItem[]>());

/**
 * 초기 피드 로드. 이미 로드된 경우 no-op.
 * userId가 주어지면 사용자 전환 감지에 사용.
 */
async function loadFeed(userId?: string): Promise<void> {
	if (loaded && loadedForUserId === (userId ?? null)) return;

	// 재진입 가드 — loading 또는 isRefreshing 중이면 no-op
	if (loading || isRefreshing) return;

	if (loadedForUserId !== (userId ?? null)) {
		feedItems = [];
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
			apiClient.fetchFeed(),
			apiClient.fetchTags(),
			apiClient.fetchMyTagIds()
		]);
		feedItems = items;
		tags = allTags;
		myTagIds = tagIds;

		// 전체 탭 + 구독 태그 전체 병렬 프리패치 → 탭 전환 항상 즉시 표시
		const tagFeedResults = await Promise.allSettled(
			tagIds.map((tagId) => apiClient.fetchFeed(tagId))
		);
		const newCache = new Map<string, FeedItem[]>([['all', items]]);
		tagIds.forEach((tagId, i) => {
			const result = tagFeedResults[i];
			if (result.status === 'fulfilled') {
				newCache.set(tagId, result.value);
			}
		});
		tagCache = newCache;

		loaded = true;
		loadedForUserId = userId ?? null;
		activeTagId = null;
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to load feed';
		feedItems = [];
	} finally {
		loading = false;
	}
}

let loadedForUserId = $state<string | null>(null);

/**
 * 탭 선택. 캐시 히트면 즉시 표시, 미스면 서버 재요청.
 * tagId: null = 전체 탭
 */
async function selectTag(tagId: string | null): Promise<void> {
	// 재진입 가드
	if (loading || isRefreshing) return;

	const key = tagId ?? 'all';

	// 캐시 히트 → 즉시 표시, 재요청 없음
	const cached = tagCache.get(key);
	if (cached !== undefined) {
		feedItems = cached;
		activeTagId = tagId;
		return;
	}

	// 캐시 미스 → 조용히 fetch (로딩 표시 없음, 프리패치 실패 시 fallback)
	try {
		const items = await apiClient.fetchFeed(tagId ?? undefined);
		feedItems = items;
		tagCache = new Map([...tagCache, [key, items]]);
		activeTagId = tagId;
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to load tag feed';
	}
}

/**
 * 새 뉴스 가져오기 — 강제 재요청. 버튼 클릭 또는 pull-to-refresh용.
 * 현재 탭 캐시를 무효화하고 재요청한다.
 * @returns 성공 시 true, 재진입/실패 시 false
 */
async function refresh(): Promise<boolean> {
	// 재진입 가드 — loading 또는 이미 isRefreshing 중이면 no-op
	if (loading || isRefreshing) return false;

	isRefreshing = true;
	error = null;

	const key = activeTagId ?? 'all';

	try {
		const items = await apiClient.fetchFeed(activeTagId ?? undefined);
		feedItems = items;
		// 캐시 갱신
		tagCache = new Map([...tagCache, [key, items]]);
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
	feedItems = [];
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
	loadFeed,
	selectTag,
	refresh,
	reset
};
