/**
 * 앱 세션 내 재작성 캐시 — url → { text, occupation } 매핑.
 * MVP16 M3 (C2-bug): occupation도 함께 캐싱.
 * occupation 변경 후 재진입 시 occupation 불일치 → 캐시 복원 안 함 (버튼 재활성화).
 * 페이지 새로고침 시 초기화됨.
 * Svelte 5 $state 기반 — 반응성 자동 트리거.
 */
export interface RewriteCacheEntry {
	text: string;
	occupation: string | null;
}

function createRewriteCache() {
	let cache = $state(new Map<string, RewriteCacheEntry>());

	return {
		get(url: string): RewriteCacheEntry | undefined {
			return cache.get(url);
		},
		set(url: string, text: string, occupation: string | null) {
			cache = new Map([...cache, [url, { text, occupation }]]);
		},
		has(url: string): boolean {
			return cache.has(url);
		},
		clear() {
			cache = new Map();
		}
	};
}

export const rewriteCache = createRewriteCache();
