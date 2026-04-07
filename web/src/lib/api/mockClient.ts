// MockApiClient — fixture JSON 기반 in-memory 구현.
// 외부 의존(Supabase, fetch) 0. M2/M3 병렬 작업 시 격리 보장.
//
// 진실의 원천 fixture: progress/fixtures/*.json
// 동기화: web/src/lib/api/__fixtures__/*.json (M1.5 시점 복사본)

import type { ApiClient } from './client';
import type {
	Article,
	FetchArticlesOptions,
	Profile,
	ProfilePatch,
	Tag
} from './types';
import articlesFixture from './__fixtures__/articles.json';
import tagsFixture from './__fixtures__/tags.json';
import profileFixture from './__fixtures__/profile.json';

// 가변 in-memory 스토어 (페이지 새로고침 시 fixture로 리셋)
let articles: Article[] = structuredClone(articlesFixture as Article[]);
const tags: Tag[] = structuredClone(tagsFixture as Tag[]);
let profile: Profile = structuredClone(profileFixture as Profile);
let myTagIds: string[] = [
	'11111111-1111-1111-1111-111111111111',
	'22222222-2222-2222-2222-222222222222'
];

// 비동기 시뮬레이션 (실제 네트워크 지연 흉내)
function delay<T>(value: T, ms = 50): Promise<T> {
	return new Promise((resolve) => setTimeout(() => resolve(value), ms));
}

export const mockApiClient: ApiClient = {
	async fetchTags(): Promise<Tag[]> {
		return delay([...tags]);
	},

	async fetchMyTagIds(): Promise<string[]> {
		return delay([...myTagIds]);
	},

	async saveMyTags(tagIds: string[]): Promise<void> {
		myTagIds = [...tagIds];
		profile = { ...profile, onboarding_completed: true };
		return delay(undefined);
	},

	async updateMyTags(tagIds: string[]): Promise<void> {
		myTagIds = [...tagIds];
		return delay(undefined);
	},

	async fetchProfile(): Promise<Profile> {
		return delay({ ...profile });
	},

	async updateProfile(patch: ProfilePatch): Promise<Profile> {
		if (patch.display_name !== undefined) {
			const trimmed = patch.display_name.trim();
			if (trimmed.length === 0) throw new Error('display_name must not be empty');
			if (trimmed.length > 50) throw new Error('display_name exceeds 50 characters');
			profile = { ...profile, display_name: trimmed };
		}
		if (patch.onboarding_completed !== undefined) {
			profile = { ...profile, onboarding_completed: patch.onboarding_completed };
		}
		return delay({ ...profile });
	},

	async fetchArticles(opts: FetchArticlesOptions = {}): Promise<Article[]> {
		const offset = opts.offset ?? 0;
		const limit = opts.limit ?? 10;
		const tagId = opts.tagId;

		let filtered = [...articles];
		if (tagId) {
			filtered = filtered.filter((a) => a.tag_id === tagId);
		}
		// created_at desc 정렬 (null은 마지막)
		filtered.sort((a, b) => {
			if (!a.created_at) return 1;
			if (!b.created_at) return -1;
			return b.created_at.localeCompare(a.created_at);
		});
		return delay(filtered.slice(offset, offset + limit));
	},

	async fetchArticleById(id: string): Promise<Article | null> {
		const found = articles.find((a) => a.id === id) ?? null;
		return delay(found);
	},

	async collectArticles(): Promise<number> {
		// 시뮬레이션: 새 가짜 기사 1건 추가
		const newArticle: Article = {
			id: crypto.randomUUID(),
			user_id: profile.id,
			tag_id: tags[0]?.id ?? null,
			title: `Mock collected article ${Date.now()}`,
			title_ko: '목업 수집 기사',
			url: `https://example.com/mock/${Date.now()}`,
			snippet: 'Mock fetched snippet',
			source: 'mock',
			search_query: 'mock',
			summary: null,
			insight: null,
			summarized_at: null,
			published_at: new Date().toISOString(),
			created_at: new Date().toISOString()
		};
		articles = [newArticle, ...articles];
		return delay(1, 200);
	},

	async summarizeArticles(): Promise<number> {
		// 시뮬레이션: 미요약 기사들에 가짜 요약 채우기
		let count = 0;
		articles = articles.map((a) => {
			if (a.summary === null) {
				count += 1;
				return {
					...a,
					summary: '목업 요약: 이 기사는 Mock 환경에서 자동 생성된 요약입니다.',
					insight: '목업 인사이트: M1.5 단계에서 사용되는 fixture 기반 데이터입니다.',
					summarized_at: new Date().toISOString()
				};
			}
			return a;
		});
		return delay(count, 200);
	}
};
