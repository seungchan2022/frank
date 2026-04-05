import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock supabase
const mockSelect = vi.fn();
const mockOrder = vi.fn();
const mockRange = vi.fn();
const mockEq = vi.fn();
const mockSingle = vi.fn();
const mockDelete = vi.fn();
const mockInsert = vi.fn();
const mockUpdate = vi.fn();

const mockFrom = vi.fn(() => ({
	select: mockSelect,
	delete: mockDelete,
	insert: mockInsert,
	update: mockUpdate
}));

const mockGetSession = vi.fn();
const mockGetUser = vi.fn();

vi.mock('$lib/supabase', () => ({
	supabase: {
		from: (table: string) => mockFrom(table),
		auth: {
			getSession: () => mockGetSession(),
			getUser: () => mockGetUser()
		}
	}
}));

import {
	fetchTags,
	fetchMyTagIds,
	saveMyTags,
	fetchProfile,
	fetchArticles,
	collectArticles,
	summarizeArticles,
	fetchArticleById,
	updateMyTags,
	getAuthHeaders
} from './api';

beforeEach(() => {
	vi.clearAllMocks();

	// Chain setup for select queries
	mockSelect.mockReturnValue({ order: mockOrder, eq: mockEq, range: mockRange });
	mockOrder.mockReturnValue({ order: mockOrder, range: mockRange, eq: mockEq });
	mockRange.mockReturnValue({ eq: mockEq, data: [], error: null });
	mockEq.mockReturnValue({ single: mockSingle, data: [], error: null });
	mockSingle.mockResolvedValue({ data: null, error: null });
	mockDelete.mockReturnValue({ eq: mockEq });
	mockInsert.mockResolvedValue({ error: null });
	mockUpdate.mockReturnValue({ eq: mockEq });
});

describe('getAuthHeaders', () => {
	it('returns auth headers when session exists', async () => {
		mockGetSession.mockResolvedValue({
			data: { session: { access_token: 'test-token' } }
		});

		const headers = await getAuthHeaders();
		expect(headers).toEqual({
			Authorization: 'Bearer test-token',
			'Content-Type': 'application/json'
		});
	});

	it('throws when no session', async () => {
		mockGetSession.mockResolvedValue({ data: { session: null } });

		await expect(getAuthHeaders()).rejects.toThrow('Not authenticated');
	});
});

describe('fetchTags', () => {
	it('returns tags from supabase', async () => {
		const tags = [{ id: '1', name: 'AI', category: 'Tech' }];
		mockOrder.mockReturnValue({ order: vi.fn().mockReturnValue({ data: tags, error: null }) });

		const result = await fetchTags();
		expect(result).toEqual(tags);
		expect(mockFrom).toHaveBeenCalledWith('tags');
	});

	it('throws on supabase error', async () => {
		mockOrder.mockReturnValue({
			order: vi.fn().mockReturnValue({ data: null, error: { message: 'DB error' } })
		});

		await expect(fetchTags()).rejects.toEqual({ message: 'DB error' });
	});
});

describe('fetchMyTagIds', () => {
	it('returns tag IDs', async () => {
		mockSelect.mockReturnValue({ data: [{ tag_id: 'a' }, { tag_id: 'b' }], error: null });

		const result = await fetchMyTagIds();
		expect(result).toEqual(['a', 'b']);
		expect(mockFrom).toHaveBeenCalledWith('user_tags');
	});

	it('throws on error', async () => {
		mockSelect.mockReturnValue({ data: null, error: { message: 'fail' } });

		await expect(fetchMyTagIds()).rejects.toEqual({ message: 'fail' });
	});
});

describe('fetchProfile', () => {
	it('returns profile data', async () => {
		const profile = { id: 'u1', display_name: 'Test', onboarding_completed: true };
		mockGetUser.mockResolvedValue({ data: { user: { id: 'u1' } } });
		mockEq.mockReturnValue({ single: vi.fn().mockResolvedValue({ data: profile, error: null }) });

		const result = await fetchProfile();
		expect(result).toEqual(profile);
	});

	it('throws when not authenticated', async () => {
		mockGetUser.mockResolvedValue({ data: { user: null } });

		await expect(fetchProfile()).rejects.toThrow('Not authenticated');
	});

	it('throws on DB query error', async () => {
		mockGetUser.mockResolvedValue({ data: { user: { id: 'u1' } } });
		mockEq.mockReturnValue({
			single: vi.fn().mockResolvedValue({ data: null, error: { message: 'DB connection lost' } })
		});

		await expect(fetchProfile()).rejects.toEqual({ message: 'DB connection lost' });
	});
});

describe('fetchArticles', () => {
	it('fetches articles with default params', async () => {
		const articles = [{ id: '1', title: 'Test' }];
		mockRange.mockReturnValue({ data: articles, error: null });

		const result = await fetchArticles();
		expect(result).toEqual(articles);
		expect(mockFrom).toHaveBeenCalledWith('articles');
	});

	it('applies tag filter', async () => {
		mockRange.mockReturnValue({ eq: vi.fn().mockReturnValue({ data: [], error: null }) });

		await fetchArticles(0, 10, 'tag-123');
		// eq should be called for tag_id filter
	});

	it('throws on error', async () => {
		mockRange.mockReturnValue({ data: null, error: { message: 'fail' } });

		await expect(fetchArticles()).rejects.toEqual({ message: 'fail' });
	});
});

describe('saveMyTags', () => {
	it('deletes old tags, inserts new, updates onboarding', async () => {
		mockGetUser.mockResolvedValue({ data: { user: { id: 'u1' } } });
		mockEq.mockReturnValue({ error: null });
		mockInsert.mockResolvedValue({ error: null });

		await saveMyTags(['t1', 't2']);
		expect(mockFrom).toHaveBeenCalledWith('user_tags');
	});

	it('throws when not authenticated', async () => {
		mockGetUser.mockResolvedValue({ data: { user: null } });

		await expect(saveMyTags(['t1'])).rejects.toThrow('Not authenticated');
	});

	it('skips insert when empty tags', async () => {
		mockGetUser.mockResolvedValue({ data: { user: { id: 'u1' } } });
		mockEq.mockReturnValue({ error: null });

		await saveMyTags([]);
		expect(mockInsert).not.toHaveBeenCalled();
	});

	it('throws when delete fails', async () => {
		mockGetUser.mockResolvedValue({ data: { user: { id: 'u1' } } });
		mockEq.mockReturnValue({ error: { message: 'delete failed' } });

		await expect(saveMyTags(['t1'])).rejects.toEqual({ message: 'delete failed' });
	});

	it('throws when onboarding update fails', async () => {
		mockGetUser.mockResolvedValue({ data: { user: { id: 'u1' } } });
		// delete succeeds, insert succeeds, but update fails
		let callCount = 0;
		mockEq.mockImplementation(() => {
			callCount++;
			if (callCount === 1) {
				// delete → eq('user_id', ...) succeeds
				return { error: null };
			}
			// update → eq('id', ...) fails
			return { error: { message: 'update failed' } };
		});
		mockInsert.mockResolvedValue({ error: null });

		await expect(saveMyTags(['t1'])).rejects.toEqual({ message: 'update failed' });
	});
});

describe('updateMyTags', () => {
	it('deletes and re-inserts tags', async () => {
		mockGetUser.mockResolvedValue({ data: { user: { id: 'u1' } } });
		mockEq.mockReturnValue({ error: null });
		mockInsert.mockResolvedValue({ error: null });

		await updateMyTags(['t1']);
		expect(mockFrom).toHaveBeenCalledWith('user_tags');
	});

	it('throws when not authenticated', async () => {
		mockGetUser.mockResolvedValue({ data: { user: null } });

		await expect(updateMyTags(['t1'])).rejects.toThrow('Not authenticated');
	});

	it('skips insert when empty tags', async () => {
		mockGetUser.mockResolvedValue({ data: { user: { id: 'u1' } } });
		mockEq.mockReturnValue({ error: null });

		await updateMyTags([]);
		expect(mockInsert).not.toHaveBeenCalled();
	});

	it('throws when delete fails', async () => {
		mockGetUser.mockResolvedValue({ data: { user: { id: 'u1' } } });
		mockEq.mockReturnValue({ error: { message: 'delete failed' } });

		await expect(updateMyTags(['t1'])).rejects.toEqual({ message: 'delete failed' });
	});

	it('throws when insert fails', async () => {
		mockGetUser.mockResolvedValue({ data: { user: { id: 'u1' } } });
		mockEq.mockReturnValue({ error: null }); // delete succeeds
		mockInsert.mockResolvedValue({ error: { message: 'insert failed' } });

		await expect(updateMyTags(['t1'])).rejects.toEqual({ message: 'insert failed' });
	});
});

describe('collectArticles', () => {
	it('calls collect endpoint and returns count', async () => {
		mockGetSession.mockResolvedValue({
			data: { session: { access_token: 'tok' } }
		});
		global.fetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ collected: 5 })
		});

		const result = await collectArticles();
		expect(result).toBe(5);
	});

	it('throws on non-ok response', async () => {
		mockGetSession.mockResolvedValue({
			data: { session: { access_token: 'tok' } }
		});
		global.fetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 500,
			json: () => Promise.resolve({ error: 'Server error' })
		});

		await expect(collectArticles()).rejects.toThrow('Server error');
	});

	it('throws Unknown error when json parse fails', async () => {
		mockGetSession.mockResolvedValue({
			data: { session: { access_token: 'tok' } }
		});
		global.fetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 500,
			json: () => Promise.reject(new Error('parse error'))
		});

		await expect(collectArticles()).rejects.toThrow('Unknown error');
	});

	it('throws fallback when error field is missing', async () => {
		mockGetSession.mockResolvedValue({
			data: { session: { access_token: 'tok' } }
		});
		global.fetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 403,
			json: () => Promise.resolve({})
		});

		await expect(collectArticles()).rejects.toThrow('Collect failed (403)');
	});
});

describe('summarizeArticles', () => {
	it('calls summarize endpoint and returns count', async () => {
		mockGetSession.mockResolvedValue({
			data: { session: { access_token: 'tok' } }
		});
		global.fetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ summarized: 3 })
		});

		const result = await summarizeArticles();
		expect(result).toBe(3);
	});

	it('throws on failure with error message', async () => {
		mockGetSession.mockResolvedValue({
			data: { session: { access_token: 'tok' } }
		});
		global.fetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 502,
			json: () => Promise.resolve({ error: 'Upstream error' })
		});

		await expect(summarizeArticles()).rejects.toThrow('Upstream error');
	});

	it('throws fallback when json parse fails', async () => {
		mockGetSession.mockResolvedValue({
			data: { session: { access_token: 'tok' } }
		});
		global.fetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 502,
			json: () => Promise.reject(new Error('parse error'))
		});

		await expect(summarizeArticles()).rejects.toThrow('Unknown error');
	});

	it('throws fallback when error field is missing', async () => {
		mockGetSession.mockResolvedValue({
			data: { session: { access_token: 'tok' } }
		});
		global.fetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 403,
			json: () => Promise.resolve({})
		});

		await expect(summarizeArticles()).rejects.toThrow('Summarize failed (403)');
	});
});

describe('fetchArticleById', () => {
	it('returns article when found', async () => {
		const article = { id: 'a1', title: 'Test' };
		mockEq.mockReturnValue({
			single: vi.fn().mockResolvedValue({ data: article, error: null })
		});

		const result = await fetchArticleById('a1');
		expect(result).toEqual(article);
	});

	it('returns null when not found', async () => {
		mockEq.mockReturnValue({
			single: vi.fn().mockResolvedValue({ data: null, error: { code: 'PGRST116' } })
		});

		const result = await fetchArticleById('nonexistent');
		expect(result).toBeNull();
	});

	it('throws on other errors', async () => {
		mockEq.mockReturnValue({
			single: vi.fn().mockResolvedValue({ data: null, error: { code: 'OTHER', message: 'fail' } })
		});

		await expect(fetchArticleById('a1')).rejects.toEqual({ code: 'OTHER', message: 'fail' });
	});
});
