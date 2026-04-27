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
	FeedItem,
	FetchArticlesOptions,
	Profile,
	ProfilePatch,
	Tag
} from './types';
import type { SummaryResult } from '$lib/types/summary';
import type { Favorite } from '$lib/types/favorite';
import type { WrongAnswer, SaveWrongAnswerBody } from '$lib/types/quiz';

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

// 401(토큰 만료) 수신 시 SvelteKit 서버 경유로 토큰 갱신.
// httpOnly 쿠키는 client JS에서 접근 불가이므로 서버 엔드포인트를 경유한다.
async function tryRefreshToken(): Promise<boolean> {
	try {
		const res = await fetch('/api/auth/token');
		if (!res.ok) return false;
		const data = (await res.json()) as { token: string | null };
		if (data.token) {
			currentAccessToken = data.token;
			return true;
		}
		return false;
	} catch {
		return false;
	}
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
	const headers = authHeaders();
	const response = await fetch(`${API_BASE}${path}`, {
		...init,
		headers: { ...headers, ...(init.headers ?? {}) }
	});

	// 401 수신 시 토큰 갱신 후 1회 재시도
	if (response.status === 401) {
		const refreshed = await tryRefreshToken();
		if (refreshed) {
			const retryHeaders = authHeaders();
			const retryResponse = await fetch(`${API_BASE}${path}`, {
				...init,
				headers: { ...retryHeaders, ...(init.headers ?? {}) }
			});
			if (!retryResponse.ok) {
				const body = (await retryResponse.json().catch(() => null)) as {
					error?: string;
				} | null;
				throw new ApiError(
					body?.error ?? `Request failed (${retryResponse.status})`,
					retryResponse.status
				);
			}
			const text = await retryResponse.text();
			return (text ? JSON.parse(text) : (undefined as unknown)) as T;
		}
	}

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

	async fetchFeed(tagId?: string, options?: { noCache?: boolean; limit?: number; offset?: number }): Promise<FeedItem[]> {
		const params = new URLSearchParams();
		if (tagId) params.set('tag_id', tagId);
		if (options?.limit !== undefined) params.set('limit', String(options.limit));
		if (options?.offset !== undefined) params.set('offset', String(options.offset));
		const qs = params.toString();
		const extraHeaders: Record<string, string> = options?.noCache
			? { 'Cache-Control': 'no-cache' }
			: {};
		return request<FeedItem[]>(`/api/me/feed${qs ? `?${qs}` : ''}`, { headers: extraHeaders });
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

	async summarize(url: string, title: string): Promise<SummaryResult> {
		try {
			return await request<SummaryResult>('/api/me/summarize', {
				method: 'POST',
				body: JSON.stringify({ url, title })
			});
		} catch (e) {
			if (e instanceof ApiError) {
				if (e.status === 422) {
					throw new Error('페이지 내용을 가져올 수 없습니다. URL을 확인해 주세요.');
				}
				if (e.status === 503) {
					throw new Error('요약 서비스를 일시적으로 사용할 수 없습니다. 잠시 후 다시 시도해 주세요.');
				}
				if (e.status === 504) {
					throw new Error('요청 시간이 초과되었습니다. 잠시 후 다시 시도해 주세요.');
				}
			}
			throw e;
		}
	},

	async addFavorite(item: FeedItem, summary?: string, insight?: string): Promise<Favorite> {
		const raw = await request<Record<string, unknown>>('/api/me/favorites', {
			method: 'POST',
			body: JSON.stringify({
				title: item.title,
				url: item.url,
				snippet: item.snippet ?? null,
				source: item.source,
				published_at: item.published_at ?? null,
				tag_id: item.tag_id ?? null,
				summary: summary ?? null,
				insight: insight ?? null,
				image_url: item.image_url ?? null
			})
		});
		return snakeToCamelFavorite(raw);
	},

	async deleteFavorite(url: string): Promise<void> {
		await request<void>(`/api/me/favorites?url=${encodeURIComponent(url)}`, {
			method: 'DELETE'
		});
	},

	async listFavorites(): Promise<Favorite[]> {
		const raw = await request<Record<string, unknown>[]>('/api/me/favorites');
		return raw.map(snakeToCamelFavorite);
	},

	async markQuizDone(url: string): Promise<void> {
		await request<void>('/api/me/favorites/quiz/done', {
			method: 'POST',
			body: JSON.stringify({ url })
		});
	},

	async saveWrongAnswer(body: SaveWrongAnswerBody): Promise<WrongAnswer> {
		const raw = await request<Record<string, unknown>>('/api/me/quiz/wrong-answers', {
			method: 'POST',
			body: JSON.stringify(body)
		});
		return snakeToCamelWrongAnswer(raw);
	},

	async listWrongAnswers(): Promise<WrongAnswer[]> {
		const raw = await request<Record<string, unknown>[]>('/api/me/quiz/wrong-answers');
		return raw.map(snakeToCamelWrongAnswer);
	},

	async deleteWrongAnswer(id: string): Promise<void> {
		await request<void>(`/api/me/quiz/wrong-answers/${encodeURIComponent(id)}`, {
			method: 'DELETE'
		});
	}
};

/// 서버 snake_case 응답 → Favorite camelCase 변환.
function snakeToCamelFavorite(raw: Record<string, unknown>): Favorite {
	return {
		id: raw.id as string,
		userId: raw.user_id as string,
		title: raw.title as string,
		url: raw.url as string,
		snippet: (raw.snippet as string | null) ?? null,
		source: raw.source as string,
		publishedAt: (raw.published_at as string | null) ?? null,
		tagId: (raw.tag_id as string | null) ?? null,
		summary: (raw.summary as string | null) ?? null,
		insight: (raw.insight as string | null) ?? null,
		likedAt: (raw.liked_at as string | null) ?? null,
		createdAt: raw.created_at as string,
		imageUrl: (raw.image_url as string | null) ?? null,
		quizCompleted: (raw.quiz_completed as boolean | null) ?? false
	};
}

/// 서버 snake_case 응답 → WrongAnswer camelCase 변환.
function snakeToCamelWrongAnswer(raw: Record<string, unknown>): WrongAnswer {
	return {
		id: raw.id as string,
		userId: raw.user_id as string,
		articleUrl: raw.article_url as string,
		articleTitle: raw.article_title as string,
		question: raw.question as string,
		options: raw.options as string[],
		correctIndex: raw.correct_index as number,
		userIndex: raw.user_index as number,
		explanation: (raw.explanation as string | null) ?? null,
		createdAt: raw.created_at as string
	};
}
