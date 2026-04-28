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
 *
 * MVP13 M2: 초기 로드 시 tag_id 기준 분리 저장 — 탭 전환 즉시 표시.
 * - loadFeed 후 items를 tag_id 별로 그룹핑해 tagCache에 완전 재분리 저장
 * - feedItems: tagCache.get(activeTagId ?? 'all')?.items 직접 참조 (클라이언트 필터링 제거)
 * - hasMore/isLoadingMore: activeTagId 기반 (태그 탭은 hasMore=false 고정)
 * - refresh: 전체 재요청 후 tagCache 완전 재구성 (stale 태그 캐시 없음)
 */

import { apiClient } from '$lib/api';
import type { FeedItem } from '$lib/types/article';
import type { Tag } from '$lib/types/tag';

/** 페이지당 기사 수 */
const PAGE_SIZE = 20;

/** 전체 탭 캐시 키. tagCache 키 하드코딩 방지. */
const ALL_TAB_KEY = 'all';

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

/** feedItems: 현재 탭의 캐시 직접 참조 (클라이언트 필터링 제거) */
let feedItems = $derived(tagCache.get(activeTagId ?? ALL_TAB_KEY)?.items ?? []);
/** hasMore: 현재 탭 기준 (태그 탭은 hasMore=false 고정) */
let hasMore = $derived(tagCache.get(activeTagId ?? ALL_TAB_KEY)?.hasMore ?? true);
/** isLoadingMore: 현재 탭 기준 */
let isLoadingMore = $derived(tagCache.get(activeTagId ?? ALL_TAB_KEY)?.status === 'loadingMore');

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
 * items를 tag_id 기준으로 그룹핑해 tagCache Map을 완전 재구성.
 * 'all' 탭 + 각 tag_id 탭 포함. 태그 탭은 hasMore=false 고정 (F-08).
 * refresh/loadFeed 공통 사용 — 항상 새 Map으로 반환해 stale 캐시 제거.
 */
function buildTagCache(items: FeedItem[]): Map<string, TabState> {
	const cache = new Map<string, TabState>([[ALL_TAB_KEY, makeTabState(items)]]);
	const grouped = new Map<string, FeedItem[]>();
	for (const item of items) {
		if (item.tag_id) {
			const existing = grouped.get(item.tag_id) ?? [];
			existing.push(item);
			grouped.set(item.tag_id, existing);
		}
	}
	for (const [tagId, groupItems] of grouped) {
		cache.set(tagId, {
			items: groupItems,
			nextOffset: groupItems.length,
			hasMore: false,
			status: 'idle'
		});
	}
	return cache;
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

		// MVP13 M2: 'all' 탭 + tag_id별 분리 저장 — 탭 전환 즉시 표시
		tagCache = buildTagCache(items);

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
 * MVP13 M2: loadFeed 시 buildTagCache로 분리 저장 완료 → 캐시 히트 보장.
 */
function selectTag(tagId: string | null): void {
	if (loading || isRefreshing) return;
	activeTagId = tagId;
}

/**
 * 다음 페이지 로드 (무한 스크롤용).
 * ALL_TAB_KEY('all') 탭 기준 — 태그 탭은 hasMore=false 고정 (F-08: 태그 탭 loadMore 없음).
 * E-02: hasMore=false 또는 isLoadingMore 중이면 재진입 차단.
 * E-03: capturedOffset으로 refresh race condition 방지.
 */
async function loadMore(): Promise<void> {
	if (isRefreshing) return;

	const key = ALL_TAB_KEY;
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
 * 새 뉴스 가져오기 — 강제 재요청. tagCache 완전 재구성.
 * loadingMore 진행 중에도 차단 — loadMore의 stale 응답이 refresh 결과를 덮어쓰는 역방향 race 방지.
 *
 * MVP13 M2: 항상 전체 재요청 후 buildTagCache로 tagCache 완전 재구성.
 * 이전에 존재했다가 사라진 태그 캐시도 자동 제거됨 (stale 없음).
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
		// MVP13 M2: 완전 재구성 — stale 태그 캐시 제거, loadMore 진행 중이던 'all' 상태 포함
		// loadMore 중인 'all' 탭 상태는 refresh 완료로 초기화 (역방향 race 방지)
		tagCache = buildTagCache(items);
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
