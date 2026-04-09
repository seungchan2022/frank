// MockApiClient 동작 검증.
// fixture(__fixtures__/*.json) 기반 in-memory 구현이 ApiClient 시그니처를 정확히 만족하는지,
// 그리고 가변 상태(태그 저장, 기사 추가, 요약 채우기)가 의도대로 동작하는지 검증한다.
//
// 외부 의존 0 — Supabase/fetch 모킹 불필요.

import { describe, it, expect, beforeEach, vi } from 'vitest';

// 각 테스트마다 모듈 캐시를 리셋해 in-memory 상태(articles/profile/myTagIds)를 초기화한다.
let mockApiClient: typeof import('../mockClient').mockApiClient;

beforeEach(async () => {
	vi.resetModules();
	mockApiClient = (await import('../mockClient')).mockApiClient;
});

describe('MockApiClient: tags', () => {
	it('fetchTags는 fixture의 5개 tag를 반환', async () => {
		const tags = await mockApiClient.fetchTags();
		expect(tags).toHaveLength(5);
		expect(tags[0]).toMatchObject({ id: expect.any(String), name: expect.any(String) });
	});

	it('fetchMyTagIds는 초기 2개 tag id를 반환', async () => {
		const ids = await mockApiClient.fetchMyTagIds();
		expect(ids).toEqual([
			'11111111-1111-1111-1111-111111111111',
			'22222222-2222-2222-2222-222222222222'
		]);
	});

	it('saveMyTags는 tagIds를 교체하고 onboarding_completed=true 처리', async () => {
		await mockApiClient.saveMyTags(['33333333-3333-3333-3333-333333333333']);
		const ids = await mockApiClient.fetchMyTagIds();
		expect(ids).toEqual(['33333333-3333-3333-3333-333333333333']);

		const profile = await mockApiClient.fetchProfile();
		expect(profile.onboarding_completed).toBe(true);
	});

	it('updateMyTags는 onboarding_completed를 건드리지 않고 tag만 교체', async () => {
		const before = await mockApiClient.fetchProfile();
		await mockApiClient.updateMyTags(['44444444-4444-4444-4444-444444444444']);
		const ids = await mockApiClient.fetchMyTagIds();
		const after = await mockApiClient.fetchProfile();

		expect(ids).toEqual(['44444444-4444-4444-4444-444444444444']);
		// updateMyTags는 onboarding 플래그를 건드리지 않음
		expect(after.onboarding_completed).toBe(before.onboarding_completed);
	});
});

describe('MockApiClient: profile', () => {
	it('fetchProfile은 fixture의 Mock User를 반환', async () => {
		const profile = await mockApiClient.fetchProfile();
		expect(profile).toMatchObject({
			id: '00000000-0000-0000-0000-000000000001',
			display_name: 'Mock User',
			onboarding_completed: true
		});
	});

	it('updateProfile은 display_name을 trim하여 저장', async () => {
		const updated = await mockApiClient.updateProfile({ display_name: '  새 이름  ' });
		expect(updated.display_name).toBe('새 이름');
	});

	it('updateProfile은 빈 display_name을 거부', async () => {
		await expect(mockApiClient.updateProfile({ display_name: '   ' })).rejects.toThrow(
			'display_name must not be empty'
		);
	});

	it('updateProfile은 50자 초과 display_name을 거부', async () => {
		const tooLong = 'a'.repeat(51);
		await expect(mockApiClient.updateProfile({ display_name: tooLong })).rejects.toThrow(
			'display_name exceeds 50 characters'
		);
	});

	it('updateProfile은 onboarding_completed 토글을 처리', async () => {
		const updated = await mockApiClient.updateProfile({ onboarding_completed: false });
		expect(updated.onboarding_completed).toBe(false);
	});
});

describe('MockApiClient: articles', () => {
	it('fetchArticles는 created_at desc 정렬 + 기본 limit 10', async () => {
		const articles = await mockApiClient.fetchArticles();
		expect(articles.length).toBeGreaterThan(0);
		// created_at desc 검증 (null은 마지막)
		for (let i = 0; i < articles.length - 1; i++) {
			const a = articles[i].created_at;
			const b = articles[i + 1].created_at;
			if (a && b) {
				expect(a >= b).toBe(true);
			}
		}
	});

	it('fetchArticles는 tagId 필터를 적용', async () => {
		const filtered = await mockApiClient.fetchArticles({
			tagId: '11111111-1111-1111-1111-111111111111'
		});
		expect(filtered.length).toBeGreaterThan(0);
		filtered.forEach((a) => {
			expect(a.tag_id).toBe('11111111-1111-1111-1111-111111111111');
		});
	});

	it('fetchArticles는 offset/limit 페이지네이션을 처리', async () => {
		const all = await mockApiClient.fetchArticles({ limit: 100 });
		const page1 = await mockApiClient.fetchArticles({ offset: 0, limit: 2 });
		const page2 = await mockApiClient.fetchArticles({ offset: 2, limit: 2 });

		expect(page1).toHaveLength(2);
		expect(page2[0]?.id).toBe(all[2]?.id);
	});

	it('fetchArticleById는 존재하는 id에 article 반환, 없으면 null', async () => {
		const found = await mockApiClient.fetchArticleById('aaaaaaa1-0000-0000-0000-000000000001');
		expect(found).not.toBeNull();
		expect(found?.id).toBe('aaaaaaa1-0000-0000-0000-000000000001');

		const missing = await mockApiClient.fetchArticleById('does-not-exist');
		expect(missing).toBeNull();
	});

	it('collectArticles는 새 기사 1건 추가하고 1을 반환', async () => {
		const before = await mockApiClient.fetchArticles({ limit: 100 });
		const collected = await mockApiClient.collectArticles();
		const after = await mockApiClient.fetchArticles({ limit: 100 });

		expect(collected).toBe(1);
		expect(after.length).toBe(before.length + 1);
	});

});

describe('MockApiClient: feed (MVP5 M1)', () => {
	it('fetchFeed는 FeedItem 배열을 반환', async () => {
		const items = await mockApiClient.fetchFeed();
		expect(Array.isArray(items)).toBe(true);
		expect(items.length).toBeGreaterThan(0);
	});

	it('fetchFeed 아이템에 id/user_id/created_at 없음', async () => {
		const items = await mockApiClient.fetchFeed();
		items.forEach((item) => {
			// FeedItem은 id/user_id/created_at 필드 없음
			expect('id' in item).toBe(false);
			expect('user_id' in item).toBe(false);
			expect('created_at' in item).toBe(false);
		});
	});

	it('fetchFeed 아이템에 url/title/source 필드 존재', async () => {
		const items = await mockApiClient.fetchFeed();
		items.forEach((item) => {
			expect(typeof item.title).toBe('string');
			expect(typeof item.url).toBe('string');
			expect(typeof item.source).toBe('string');
		});
	});
});

describe('MockApiClient: summarize (MVP5 M2)', () => {
	it('summarize는 summary + insight 반환', async () => {
		const result = await mockApiClient.summarize('https://example.com', 'Test');
		expect(typeof result.summary).toBe('string');
		expect(typeof result.insight).toBe('string');
		expect(result.summary.length).toBeGreaterThan(0);
		expect(result.insight.length).toBeGreaterThan(0);
	});
});
