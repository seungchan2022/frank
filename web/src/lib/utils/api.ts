import { supabase } from '$lib/supabase';
import type { Tag } from '$lib/types/tag';
import type { Article } from '$lib/types/article';

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

export async function fetchTags(): Promise<Tag[]> {
	const { data, error } = await supabase.from('tags').select('id, name, category').order('category').order('name');
	if (error) throw error;
	return data;
}

export async function fetchMyTagIds(): Promise<string[]> {
	const { data, error } = await supabase.from('user_tags').select('tag_id');
	if (error) throw error;
	return data.map((row: { tag_id: string }) => row.tag_id);
}

export async function saveMyTags(tagIds: string[]): Promise<void> {
	const {
		data: { user }
	} = await supabase.auth.getUser();
	if (!user) throw new Error('Not authenticated');

	// 기존 태그 삭제
	const { error: delError } = await supabase.from('user_tags').delete().eq('user_id', user.id);
	if (delError) throw delError;

	// 새 태그 삽입
	if (tagIds.length > 0) {
		const rows = tagIds.map((tag_id) => ({ user_id: user.id, tag_id }));
		const { error: insError } = await supabase.from('user_tags').insert(rows);
		if (insError) throw insError;
	}

	// 온보딩 완료 처리
	const { error: updError } = await supabase
		.from('profiles')
		.update({ onboarding_completed: true })
		.eq('id', user.id);
	if (updError) throw updError;
}

export async function fetchProfile() {
	const {
		data: { user }
	} = await supabase.auth.getUser();
	if (!user) throw new Error('Not authenticated');

	const { data, error } = await supabase
		.from('profiles')
		.select('id, display_name, onboarding_completed')
		.eq('id', user.id)
		.single();
	if (error) throw error;
	return data;
}

export async function fetchArticles(): Promise<Article[]> {
	const { data, error } = await supabase
		.from('articles')
		.select(
			'id, user_id, tag_id, title, url, snippet, source, search_query, published_at, created_at, summary, insight, summarized_at'
		)
		.order('created_at', { ascending: false })
		.limit(50);
	if (error) throw error;
	return data;
}

/**
 * Placeholder for article collection trigger.
 * Will be wired to Rust server POST /api/collect in the future.
 * Returns the number of articles collected.
 */
export async function collectArticles(): Promise<number> {
	// TODO: POST to Rust server /api/collect when ready
	return 0;
}

/**
 * Placeholder for LLM summarization trigger.
 * Will be wired to Rust server POST /api/summarize in the future.
 * Returns the number of articles summarized.
 */
export async function summarizeArticles(): Promise<number> {
	// TODO: POST to Rust server /api/summarize when ready
	return 0;
}

/**
 * Fetch a single article by ID.
 */
export async function fetchArticleById(id: string): Promise<Article | null> {
	const { data, error } = await supabase
		.from('articles')
		.select(
			'id, user_id, tag_id, title, url, snippet, source, search_query, published_at, created_at, summary, insight, summarized_at'
		)
		.eq('id', id)
		.single();
	if (error) {
		if (error.code === 'PGRST116') return null; // not found
		throw error;
	}
	return data;
}

/**
 * Update user tags without touching onboarding status.
 * Used by the settings page.
 */
export async function updateMyTags(tagIds: string[]): Promise<void> {
	const {
		data: { user }
	} = await supabase.auth.getUser();
	if (!user) throw new Error('Not authenticated');

	const { error: delError } = await supabase.from('user_tags').delete().eq('user_id', user.id);
	if (delError) throw delError;

	if (tagIds.length > 0) {
		const rows = tagIds.map((tag_id) => ({ user_id: user.id, tag_id }));
		const { error: insError } = await supabase.from('user_tags').insert(rows);
		if (insError) throw insError;
	}
}

// getAuthHeaders는 Rust 서버 프록시용으로 예비
export { getAuthHeaders };
