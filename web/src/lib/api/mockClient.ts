// MockApiClient — fixture JSON 기반 in-memory 구현.
// 외부 의존(Supabase, fetch) 0. M2/M3 병렬 작업 시 격리 보장.
//
// 진실의 원천 fixture: progress/fixtures/*.json
// 동기화: web/src/lib/api/__fixtures__/*.json (M1.5 시점 복사본)

import type { ApiClient } from './client';
import type {
	Article,
	FeedItem,
	FetchArticlesOptions,
	Profile,
	ProfilePatch,
	Tag
} from './types';
import type { SummaryResult } from '$lib/types/summary';
import type { Favorite } from '$lib/types/favorite';
import type { WrongAnswer, SaveWrongAnswerBody } from '$lib/types/quiz';
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

	async fetchFeed(_tagId?: string, _options?: { noCache?: boolean; limit?: number; offset?: number }): Promise<FeedItem[]> {
		// Mock: 현재 articles fixture를 FeedItem 형태로 변환 (ephemeral 시뮬레이션)
		let feedItems: FeedItem[] = articles.map((a) => ({
			title: a.title,
			url: a.url,
			snippet: a.snippet,
			source: a.source,
			published_at: a.published_at,
			tag_id: a.tag_id
		}));
		// limit/offset 페이지네이션 시뮬레이션
		const offset = _options?.offset ?? 0;
		const limit = _options?.limit ?? feedItems.length;
		feedItems = feedItems.slice(offset, offset + limit);
		return delay([...feedItems]);
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
			url: `https://example.com/mock/${Date.now()}`,
			snippet: 'Mock fetched snippet',
			source: 'mock',
			published_at: new Date().toISOString(),
			created_at: new Date().toISOString()
		};
		articles = [newArticle, ...articles];
		return delay(1, 200);
	},

	async summarize(_url: string, _title: string): Promise<SummaryResult> {
		return delay(
			{
				summary: 'Mock 요약: 이 기사는 AI 기술의 최신 동향을 다루고 있습니다.',
				insight: 'Mock 인사이트: AI 기술이 산업 전반에 미치는 영향이 커지고 있습니다.'
			},
			600
		);
	},

	async addFavorite(item: FeedItem, summary?: string, insight?: string): Promise<Favorite> {
		const existing = mockFavorites.find((f) => f.url === item.url);
		if (existing) throw Object.assign(new Error('이미 즐겨찾기에 추가된 기사입니다.'), { status: 409 });
		const now = new Date().toISOString();
		const fav: Favorite = {
			id: crypto.randomUUID(),
			userId: profile.id,
			title: item.title,
			url: item.url,
			snippet: item.snippet ?? null,
			source: item.source,
			publishedAt: item.published_at ?? null,
			tagId: item.tag_id ?? null,
			summary: summary ?? null,
			insight: insight ?? null,
			likedAt: now,
			createdAt: now
		};
		mockFavorites = [fav, ...mockFavorites];
		return delay({ ...fav });
	},

	async deleteFavorite(url: string): Promise<void> {
		mockFavorites = mockFavorites.filter((f) => f.url !== url);
		return delay(undefined);
	},

	async listFavorites(): Promise<Favorite[]> {
		return delay([...mockFavorites]);
	},

	async markQuizDone(url: string): Promise<void> {
		mockFavorites = mockFavorites.map((f) =>
			f.url === url ? { ...f, quizCompleted: true } : f
		);
		return delay(undefined);
	},

	async saveWrongAnswer(body: SaveWrongAnswerBody): Promise<WrongAnswer> {
		const now = new Date().toISOString();
		const wa: WrongAnswer = {
			id: crypto.randomUUID(),
			userId: profile.id,
			articleUrl: body.article_url,
			articleTitle: body.article_title,
			question: body.question,
			options: body.options,
			correctIndex: body.correct_index,
			userIndex: body.user_index,
			explanation: body.explanation,
			createdAt: now
		};
		mockWrongAnswers = [wa, ...mockWrongAnswers];
		return delay({ ...wa });
	},

	async listWrongAnswers(): Promise<WrongAnswer[]> {
		return delay([...mockWrongAnswers]);
	},

	async deleteWrongAnswer(id: string): Promise<void> {
		mockWrongAnswers = mockWrongAnswers.filter((wa) => wa.id !== id);
		return delay(undefined);
	}
};

// 인메모리 즐겨찾기 스토어
let mockFavorites: Favorite[] = [];
// 인메모리 오답 스토어
let mockWrongAnswers: WrongAnswer[] = [];
