// ApiClient 인터페이스 — 진실의 원천: progress/260407_API_SPEC.md
//
// Mock/Real 두 구현체는 이 인터페이스를 준수해야 한다.
// M2(웹 전환)에서 RealApiClient의 내부 구현이 Supabase 직접 호출 → Rust API로 교체된다.
// 컴포넌트는 이 인터페이스만 의존한다.

import type {
	Article,
	FetchArticlesOptions,
	Profile,
	ProfilePatch,
	Tag
} from './types';

export interface ApiClient {
	// Tags
	fetchTags(): Promise<Tag[]>;
	fetchMyTagIds(): Promise<string[]>;
	saveMyTags(tagIds: string[]): Promise<void>;
	updateMyTags(tagIds: string[]): Promise<void>;

	// Profile
	fetchProfile(): Promise<Profile>;
	updateProfile(patch: ProfilePatch): Promise<Profile>;

	// Articles
	fetchArticles(opts?: FetchArticlesOptions): Promise<Article[]>;
	fetchArticleById(id: string): Promise<Article | null>;

	// Pipeline
	collectArticles(): Promise<number>;
	summarizeArticles(): Promise<number>;
}
