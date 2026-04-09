import type { SummaryResult } from '$lib/types/summary';

/**
 * 앱 세션 내 요약 캐시 — url → SummaryResult 매핑.
 * 페이지 새로고침 시 초기화됨.
 * Svelte 5 $state 기반 — 반응성 자동 트리거.
 */
function createSummaryCache() {
	let cache = $state(new Map<string, SummaryResult>());

	return {
		get(url: string): SummaryResult | undefined {
			return cache.get(url);
		},
		set(url: string, result: SummaryResult) {
			cache = new Map([...cache, [url, result]]);
		},
		has(url: string): boolean {
			return cache.has(url);
		},
		clear() {
			cache = new Map();
		}
	};
}

export const summaryCache = createSummaryCache();

export function getSummaryCache() {
	return summaryCache;
}
