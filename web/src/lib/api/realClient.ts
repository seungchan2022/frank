// RealApiClient — 현재 동작 그대로 (Supabase 직접 호출 + Rust 프록시).
// M2에서 Supabase 직접 호출 부분이 fetch(Rust API)로 교체된다.

import { supabase } from '$lib/supabase';
import type { ApiClient } from './client';
import type {
	Article,
	FetchArticlesOptions,
	Profile,
	ProfilePatch,
	Tag
} from './types';

const PAGE_SIZE = 10;
const ARTICLE_SELECT =
	'id, user_id, tag_id, title, title_ko, url, snippet, source, search_query, published_at, created_at, summary, insight, summarized_at';

async function getAuthHeaders(): Promise<Record<string, string>> {
	const {
		data: { session }
	} = await supabase.auth.getSession();
	if (!session) throw new Error('Not authenticated');
	return {
		Authorization: `Bearer ${session.access_token}`,
		'Content-Type': 'application/json'
	};
}

async function currentUserId(): Promise<string> {
	const {
		data: { user }
	} = await supabase.auth.getUser();
	if (!user) throw new Error('Not authenticated');
	return user.id;
}

export const realApiClient: ApiClient = {
	async fetchTags(): Promise<Tag[]> {
		const { data, error } = await supabase
			.from('tags')
			.select('id, name, category')
			.order('category')
			.order('name');
		if (error) throw error;
		return data as Tag[];
	},

	async fetchMyTagIds(): Promise<string[]> {
		const { data, error } = await supabase.from('user_tags').select('tag_id');
		if (error) throw error;
		return data.map((row: { tag_id: string }) => row.tag_id);
	},

	async saveMyTags(tagIds: string[]): Promise<void> {
		const userId = await currentUserId();

		// 기존 태그 삭제
		const { error: delError } = await supabase.from('user_tags').delete().eq('user_id', userId);
		if (delError) throw delError;

		// 새 태그 삽입
		if (tagIds.length > 0) {
			const rows = tagIds.map((tag_id) => ({ user_id: userId, tag_id }));
			const { error: insError } = await supabase.from('user_tags').insert(rows);
			if (insError) throw insError;
		}

		// 온보딩 완료 처리
		const { error: updError } = await supabase
			.from('profiles')
			.update({ onboarding_completed: true })
			.eq('id', userId);
		if (updError) throw updError;
	},

	async updateMyTags(tagIds: string[]): Promise<void> {
		const userId = await currentUserId();

		const { error: delError } = await supabase.from('user_tags').delete().eq('user_id', userId);
		if (delError) throw delError;

		if (tagIds.length > 0) {
			const rows = tagIds.map((tag_id) => ({ user_id: userId, tag_id }));
			const { error: insError } = await supabase.from('user_tags').insert(rows);
			if (insError) throw insError;
		}
	},

	async fetchProfile(): Promise<Profile> {
		const userId = await currentUserId();
		const { data, error } = await supabase
			.from('profiles')
			.select('id, display_name, onboarding_completed')
			.eq('id', userId)
			.single();
		if (error) throw error;
		return data as Profile;
	},

	async updateProfile(patch: ProfilePatch): Promise<Profile> {
		const userId = await currentUserId();
		const update: Record<string, unknown> = {};
		if (patch.display_name !== undefined) update.display_name = patch.display_name;
		if (patch.onboarding_completed !== undefined)
			update.onboarding_completed = patch.onboarding_completed;

		// 빈 패치는 현재 프로필 그대로 반환
		if (Object.keys(update).length === 0) {
			return this.fetchProfile();
		}

		const { data, error } = await supabase
			.from('profiles')
			.update(update)
			.eq('id', userId)
			.select('id, display_name, onboarding_completed')
			.single();
		if (error) throw error;
		return data as Profile;
	},

	async fetchArticles(opts: FetchArticlesOptions = {}): Promise<Article[]> {
		const offset = opts.offset ?? 0;
		const limit = opts.limit ?? PAGE_SIZE;
		const tagId = opts.tagId;

		let query = supabase
			.from('articles')
			.select(ARTICLE_SELECT)
			.order('created_at', { ascending: false })
			.range(offset, offset + limit - 1);

		if (tagId) {
			query = query.eq('tag_id', tagId);
		}

		const { data, error } = await query;
		if (error) throw error;
		return data as Article[];
	},

	async fetchArticleById(id: string): Promise<Article | null> {
		const { data, error } = await supabase
			.from('articles')
			.select(ARTICLE_SELECT)
			.eq('id', id)
			.single();
		if (error) {
			if (error.code === 'PGRST116') return null; // not found
			throw error;
		}
		return data as Article;
	},

	async collectArticles(): Promise<number> {
		const headers = await getAuthHeaders();
		const response = await fetch('/api/collect', { method: 'POST', headers });
		if (!response.ok) {
			const body = await response.json().catch(() => ({ error: 'Unknown error' }));
			throw new Error(body.error ?? `Collect failed (${response.status})`);
		}
		const data: { collected: number } = await response.json();
		return data.collected;
	},

	async summarizeArticles(): Promise<number> {
		const headers = await getAuthHeaders();
		const response = await fetch('/api/summarize', { method: 'POST', headers });
		if (!response.ok) {
			const body = await response.json().catch(() => ({ error: 'Unknown error' }));
			throw new Error(body.error ?? `Summarize failed (${response.status})`);
		}
		const data: { summarized: number } = await response.json();
		return data.summarized;
	}
};
