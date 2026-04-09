// summaryCache.svelte.ts 단위 테스트
// Svelte 5 $state 기반 캐시 — get/set/has/clear 검증.

import { describe, it, expect, beforeEach, vi } from 'vitest';

let summaryCache: typeof import('./summaryCache.svelte').summaryCache;

// 각 테스트 전 모듈 리셋 (in-memory 상태 초기화)
beforeEach(async () => {
	vi.resetModules();
	const mod = await import('./summaryCache.svelte');
	summaryCache = mod.summaryCache;
});

describe('summaryCache', () => {
	it('초기 상태: 빈 캐시', () => {
		expect(summaryCache.has('https://example.com')).toBe(false);
		expect(summaryCache.get('https://example.com')).toBeUndefined();
	});

	it('set 후 get → 동일 결과 반환', () => {
		const result = { summary: 'Test summary', insight: 'Test insight' };
		summaryCache.set('https://example.com/article', result);

		expect(summaryCache.has('https://example.com/article')).toBe(true);
		expect(summaryCache.get('https://example.com/article')).toEqual(result);
	});

	it('set 후 다른 url로 get → undefined', () => {
		summaryCache.set('https://a.com', { summary: 'a', insight: 'a' });
		expect(summaryCache.get('https://b.com')).toBeUndefined();
	});

	it('같은 url 재설정 → 최신 값으로 덮어쓰기', () => {
		summaryCache.set('https://example.com', { summary: 'old', insight: 'old' });
		summaryCache.set('https://example.com', { summary: 'new', insight: 'new' });

		const result = summaryCache.get('https://example.com');
		expect(result?.summary).toBe('new');
	});

	it('clear 후 캐시 비어있음', () => {
		summaryCache.set('https://a.com', { summary: 'a', insight: 'a' });
		summaryCache.set('https://b.com', { summary: 'b', insight: 'b' });

		summaryCache.clear();

		expect(summaryCache.has('https://a.com')).toBe(false);
		expect(summaryCache.has('https://b.com')).toBe(false);
	});

	it('여러 url 독립적으로 저장', () => {
		summaryCache.set('https://a.com', { summary: 'sum a', insight: 'ins a' });
		summaryCache.set('https://b.com', { summary: 'sum b', insight: 'ins b' });

		expect(summaryCache.get('https://a.com')?.summary).toBe('sum a');
		expect(summaryCache.get('https://b.com')?.summary).toBe('sum b');
	});
});
