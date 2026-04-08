// RealApiClient — Rust API HTTP 클라이언트.
//
// 모든 데이터 호출은 fetch(VITE_RUST_API_URL/...) + Bearer 토큰.
// 토큰은 +layout.svelte의 setRealClientToken(...)로 주입된다.
// 진실의 원천: progress/260407_API_SPEC.md
//
// M2 ST-4 (옵션 B): cookie httpOnly 보장 → client-side supabase.auth.getSession 사용 불가.
// page.data.session.access_token을 +layout.svelte가 주입해야 한다.

import type { ApiClient } from './client';
import type {
	Article,
	FetchArticlesOptions,
	Profile,
	ProfilePatch,
	Tag
} from './types';

const API_BASE = (import.meta.env.VITE_RUST_API_URL ?? 'http://localhost:8080').replace(/\/$/, '');

let currentAccessToken: string | null = null;

/**
 * +layout.svelte의 $effect에서 page.data.session 변경 시 호출.
 * realClient 호출 시점에 최신 access_token을 사용한다.
 */
export function setRealClientToken(token: string | null): void {
	currentAccessToken = token;
}

function authHeaders(): Record<string, string> {
	if (!currentAccessToken) throw new Error('Not authenticated');
	return {
		Authorization: `Bearer ${currentAccessToken}`,
		'Content-Type': 'application/json'
	};
}

class ApiError extends Error {
	constructor(
		message: string,
		readonly status: number
	) {
		super(message);
		this.name = 'ApiError';
	}
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
	const headers = authHeaders();
	const response = await fetch(`${API_BASE}${path}`, {
		...init,
		headers: { ...headers, ...(init.headers ?? {}) }
	});
	if (!response.ok) {
		const body = (await response.json().catch(() => null)) as { error?: string } | null;
		throw new ApiError(
			body?.error ?? `Request failed (${response.status})`,
			response.status
		);
	}
	// 204 No Content 가능성 대비
	const text = await response.text();
	return (text ? JSON.parse(text) : (undefined as unknown)) as T;
}

export const realApiClient: ApiClient = {
	async fetchTags(): Promise<Tag[]> {
		return request<Tag[]>('/api/tags');
	},

	async fetchMyTagIds(): Promise<string[]> {
		return request<string[]>('/api/me/tags');
	},

	async saveMyTags(tagIds: string[]): Promise<void> {
		// POST /api/me/tags — 부수효과로 onboarding_completed = true
		await request<{ ok: boolean }>('/api/me/tags', {
			method: 'POST',
			body: JSON.stringify({ tag_ids: tagIds })
		});
	},

	async updateMyTags(tagIds: string[]): Promise<void> {
		// 동일 엔드포인트. 설정 페이지에서는 onboarding_completed가 이미 true이므로 무해.
		await request<{ ok: boolean }>('/api/me/tags', {
			method: 'POST',
			body: JSON.stringify({ tag_ids: tagIds })
		});
	},

	async fetchProfile(): Promise<Profile> {
		return request<Profile>('/api/me/profile');
	},

	async updateProfile(patch: ProfilePatch): Promise<Profile> {
		return request<Profile>('/api/me/profile', {
			method: 'PUT',
			body: JSON.stringify(patch)
		});
	},

	async fetchArticles(opts: FetchArticlesOptions = {}): Promise<Article[]> {
		const params = new URLSearchParams();
		if (opts.offset !== undefined) params.set('offset', String(opts.offset));
		if (opts.limit !== undefined) params.set('limit', String(opts.limit));
		if (opts.tagId) params.set('tag_id', opts.tagId);
		const qs = params.toString();
		return request<Article[]>(`/api/me/articles${qs ? `?${qs}` : ''}`);
	},

	async fetchArticleById(id: string): Promise<Article | null> {
		try {
			return await request<Article>(`/api/me/articles/${encodeURIComponent(id)}`);
		} catch (e) {
			// 404는 null 반환 (SPEC: 본인 외 기사 또는 없는 기사 통일)
			if (e instanceof ApiError && e.status === 404) return null;
			throw e;
		}
	},

	async collectArticles(): Promise<number> {
		const data = await request<{ collected: number }>('/api/me/collect', { method: 'POST' });
		return data.collected;
	},

	async summarizeArticles(signal?: AbortSignal): Promise<number> {
		const data = await request<{ summarized: number }>('/api/me/summarize', {
			method: 'POST',
			signal
		});
		return data.summarized;
	}
};
