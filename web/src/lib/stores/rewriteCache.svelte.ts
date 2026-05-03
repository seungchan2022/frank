/**
 * 앱 세션 내 재작성 캐시 — url → rewrite 텍스트 매핑.
 * 페이지 새로고침 시 초기화됨.
 * Svelte 5 $state 기반 — 반응성 자동 트리거.
 */
function createRewriteCache() {
	let cache = $state(new Map<string, string>());

	return {
		get(url: string): string | undefined {
			return cache.get(url);
		},
		set(url: string, rewrite: string) {
			cache = new Map([...cache, [url, rewrite]]);
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
