/**
 * MVP5 M3: 피드 상태 관리 store.
 * Svelte 5 $state 기반 모듈 수준 상태 — 페이지 remount(뒤로가기 복귀) 시에도 데이터 유지.
 * SvelteKit 라우터가 +page.svelte를 재마운트해도 피드를 재요청하지 않는다.
 */

import { apiClient } from '$lib/api';
import type { FeedItem } from '$lib/types/article';
import type { Tag } from '$lib/types/tag';

let feedItems = $state<FeedItem[]>([]);
let tags = $state<Tag[]>([]);
let myTagIds = $state<string[]>([]);
let loaded = $state(false);
let loading = $state(false);
let error = $state<string | null>(null);

/**
 * 초기 피드 로드. 이미 로드된 경우 no-op.
 * userId가 주어지면 사용자 전환 감지에 사용.
 */
async function loadFeed(userId?: string): Promise<void> {
	if (loaded && loadedForUserId === (userId ?? null)) return;

	if (loadedForUserId !== (userId ?? null)) {
		feedItems = [];
		tags = [];
		myTagIds = [];
		loaded = false;
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
		loaded = true;
		loadedForUserId = userId ?? null;
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to load feed';
		feedItems = [];
	} finally {
		loading = false;
	}
}

let loadedForUserId = $state<string | null>(null);

/**
 * 새 뉴스 가져오기 — 강제 재요청. 버튼 클릭 또는 pull-to-refresh용.
 */
async function refresh(): Promise<void> {
	loading = true;
	error = null;

	try {
		feedItems = await apiClient.fetchFeed();
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to refresh feed';
	} finally {
		loading = false;
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
	error = null;
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
	get error() {
		return error;
	},
	loadFeed,
	refresh,
	reset
};
