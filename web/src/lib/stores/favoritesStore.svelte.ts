/**
 * MVP5 M3: 즐겨찾기 상태 관리 store.
 * Svelte 5 $state 기반 — 반응성 자동 트리거.
 *
 * step-5 필수 수정 H 반영:
 * - `loaded: boolean` 플래그 + `loadedForUserId` 체크로 빈 즐겨찾기/사용자 전환 오염 방지.
 * - `if favorites.length > 0 return` guard는 사용하지 않음.
 */

import { apiClient } from '$lib/api';
import type { FeedItem } from '$lib/api/types';
import type { Favorite } from '$lib/types/favorite';

let favorites = $state<Favorite[]>([]);
let loaded = $state(false);
let loadedForUserId = $state<string | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);

// $derived: favorites에서 url Set 추출
let likedUrls = $derived(new Set(favorites.map((f) => f.url)));

/**
 * 즐겨찾기 목록 로드.
 * 동일 userId로 이미 로드된 경우 no-op.
 * userId가 바뀌면 상태 초기화 후 재로드.
 */
async function loadFavorites(userId?: string): Promise<void> {
	// 같은 사용자로 이미 로드됐으면 no-op
	if (loaded && loadedForUserId === (userId ?? null)) return;

	// 사용자 전환 시 상태 초기화
	if (loadedForUserId !== (userId ?? null)) {
		favorites = [];
		loaded = false;
	}

	loading = true;
	error = null;

	try {
		const result = await apiClient.listFavorites();
		favorites = result;
		loaded = true;
		loadedForUserId = userId ?? null;
	} catch (e) {
		error = e instanceof Error ? e.message : '즐겨찾기를 불러오지 못했습니다.';
	} finally {
		loading = false;
	}
}

/**
 * 즐겨찾기 추가.
 * Svelte 5 반응성 규칙: 새 배열 할당.
 */
async function addFavorite(item: FeedItem, summary?: string, insight?: string): Promise<void> {
	const added = await apiClient.addFavorite(item, summary, insight);
	// prepend — DESC 순서 유지
	favorites = [added, ...favorites];
}

/**
 * 즐겨찾기 삭제.
 * Svelte 5 반응성 규칙: 새 배열 할당.
 */
async function removeFavorite(url: string): Promise<void> {
	await apiClient.deleteFavorite(url);
	favorites = favorites.filter((f) => f.url !== url);
}

/**
 * 즐겨찾기 여부 확인.
 */
function isLiked(url: string): boolean {
	return likedUrls.has(url);
}

/**
 * 상태 완전 초기화 (로그아웃 등 세션 전환 시).
 */
function reset(): void {
	favorites = [];
	loaded = false;
	loadedForUserId = null;
	loading = false;
	error = null;
}

export const favoritesStore = {
	get favorites() {
		return favorites;
	},
	get likedUrls() {
		return likedUrls;
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
	isLiked,
	loadFavorites,
	addFavorite,
	removeFavorite,
	reset
};
