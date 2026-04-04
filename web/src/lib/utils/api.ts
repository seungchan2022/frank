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

const PAGE_SIZE = 10;

export async function fetchArticles(offset = 0, limit = PAGE_SIZE): Promise<Article[]> {
	const { data, error } = await supabase
		.from('articles')
		.select(
			'id, user_id, tag_id, title, url, snippet, source, search_query, published_at, created_at, summary, insight, summarized_at'
		)
		.order('created_at', { ascending: false })
		.range(offset, offset + limit - 1);
	if (error) throw error;
	return data;
}

/**
 * Trigger article collection via the SvelteKit proxy → Rust server.
 * Returns the number of articles collected.
 */
export async function collectArticles(): Promise<number> {
	const headers = await getAuthHeaders();
	const response = await fetch('/api/collect', {
		method: 'POST',
		headers
	});
	if (!response.ok) {
		const body = await response.json().catch(() => ({ error: 'Unknown error' }));
		throw new Error(body.error ?? `Collect failed (${response.status})`);
	}
	const data: { collected: number } = await response.json();
	return data.collected;
}

/**
 * Trigger LLM summarization via the SvelteKit proxy → Rust server.
 * Returns the number of articles summarized.
 */
export async function summarizeArticles(): Promise<number> {
	const headers = await getAuthHeaders();
	const response = await fetch('/api/summarize', {
		method: 'POST',
		headers
	});
	if (!response.ok) {
		const body = await response.json().catch(() => ({ error: 'Unknown error' }));
		throw new Error(body.error ?? `Summarize failed (${response.status})`);
	}
	const data: { summarized: number } = await response.json();
	return data.summarized;
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
