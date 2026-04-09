// feed/+page.svelte 피드 UI 테스트 (MVP5 M1)
// 요약 제거 후 카드에 제목 + source + 날짜만 표시되는지 확인.
// 새로고침 버튼 → collect 후 목록 갱신 UX 확인.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

// ── IntersectionObserver mock (jsdom 미구현, new 가능한 class로) ──────────
global.IntersectionObserver = class {
	observe = vi.fn();
	unobserve = vi.fn();
	disconnect = vi.fn();
} as unknown as typeof IntersectionObserver;

// ── SvelteKit 모듈 mock ───────────────────────────────────────────────────
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({ page: { url: { pathname: '/feed' } } }));
vi.mock('$app/forms', () => ({ enhance: () => ({ destroy: () => undefined }) }));

// ── auth store mock (항상 로그인 상태) ────────────────────────────────────
vi.mock('$lib/stores/auth.svelte', () => ({
	getAuth: () => ({ isAuthenticated: true, loading: false, user: { email: 't@t.com' } })
}));

// ── apiClient mock ────────────────────────────────────────────────────────
const mockCollectArticles = vi.fn<() => Promise<number>>();
const mockFetchArticles = vi.fn();
const mockFetchTags = vi.fn();
const mockFetchMyTagIds = vi.fn();

vi.mock('$lib/api', () => ({
	apiClient: {
		collectArticles: () => mockCollectArticles(),
		fetchArticles: (...args: unknown[]) => mockFetchArticles(...args),
		fetchTags: () => mockFetchTags(),
		fetchMyTagIds: () => mockFetchMyTagIds()
	}
}));

import FeedPage from '../+page.svelte';

const sampleArticles = [
	{
		id: 'article-1',
		user_id: 'user-1',
		tag_id: null,
		title: 'AI Revolution 2026',
		url: 'https://example.com/article-1',
		snippet: 'AI is changing everything.',
		source: 'TechCrunch',
		published_at: '2026-04-09T10:00:00Z',
		created_at: '2026-04-09T10:00:00Z'
	}
];

beforeEach(() => {
	vi.clearAllMocks();

	// 기본: 즉시 완료
	mockFetchArticles.mockResolvedValue([]);
	mockFetchTags.mockResolvedValue([]);
	mockFetchMyTagIds.mockResolvedValue([]);
	mockCollectArticles.mockResolvedValue(1);
});

afterEach(() => {
	cleanup();
});

describe('feed/+page.svelte — MVP5 M1 피드 UI', () => {
	it('초기 상태: 빈 피드 메시지 표시', async () => {
		render(FeedPage);

		await waitFor(() => {
			expect(screen.queryByText(/No articles yet/)).toBeTruthy();
		});
	});

	it('기사 카드: 제목 표시', async () => {
		mockFetchArticles.mockResolvedValue(sampleArticles);

		render(FeedPage);

		await waitFor(() => {
			expect(screen.getByText('AI Revolution 2026')).toBeTruthy();
		});
	});

	it('기사 카드: source 표시', async () => {
		mockFetchArticles.mockResolvedValue(sampleArticles);

		render(FeedPage);

		await waitFor(() => {
			expect(screen.getByText('TechCrunch')).toBeTruthy();
		});
	});

	it('기사 카드: 요약(summary) 미표시 — 제목+source+날짜만', async () => {
		mockFetchArticles.mockResolvedValue(sampleArticles);

		render(FeedPage);

		await waitFor(() => {
			// 요약 관련 텍스트 없음
			expect(screen.queryByText(/요약/)).toBeNull();
			expect(screen.queryByText(/Summarize/i)).toBeNull();
		});
	});

	it('새 뉴스 가져오기 버튼 존재', async () => {
		render(FeedPage);

		await waitFor(() => {
			expect(screen.getByRole('button', { name: /새 뉴스 가져오기/i })).toBeTruthy();
		});
	});

	it('새 뉴스 가져오기 버튼 클릭 → collectArticles 호출', async () => {
		mockCollectArticles.mockResolvedValue(1);
		mockFetchArticles.mockResolvedValue([]);

		render(FeedPage);
		await waitFor(() => {
			expect(screen.queryByText(/Loading/)).toBeNull();
		});

		const btn = screen.getByRole('button', { name: /새 뉴스 가져오기/i });
		fireEvent.click(btn);

		await waitFor(() => {
			expect(mockCollectArticles).toHaveBeenCalledTimes(1);
		});
	});

	it('수집 중 배너 표시', async () => {
		// collectArticles가 느리게 완료
		mockCollectArticles.mockImplementation(
			() => new Promise((resolve) => setTimeout(() => resolve(1), 200))
		);
		mockFetchArticles.mockResolvedValue([]);

		render(FeedPage);
		await waitFor(() => {
			expect(screen.queryByText(/Loading/)).toBeNull();
		});

		const btn = screen.getByRole('button', { name: /새 뉴스 가져오기/i });
		fireEvent.click(btn);

		await waitFor(() => {
			expect(screen.getByText(/수집하고 있어요/)).toBeTruthy();
		});
	});

	it('수집 완료 후 기사 목록 갱신', async () => {
		const newArticle = { ...sampleArticles[0], id: 'new-article', title: 'New Article After Collect' };
		mockCollectArticles.mockResolvedValue(1);
		mockFetchArticles
			.mockResolvedValueOnce([]) // loadInitial
			.mockResolvedValueOnce([newArticle]) // handleRefresh 후 fetch
			.mockResolvedValueOnce([]); // fetchTags는 별도

		mockFetchTags.mockResolvedValue([]);

		render(FeedPage);
		await waitFor(() => {
			expect(screen.queryByText(/Loading/)).toBeNull();
		});

		const btn = screen.getByRole('button', { name: /새 뉴스 가져오기/i });
		fireEvent.click(btn);

		await waitFor(() => {
			expect(screen.getByText('New Article After Collect')).toBeTruthy();
		});
	});

	it('수집 실패 시 에러 메시지 표시', async () => {
		mockCollectArticles.mockRejectedValue(new Error('Network error'));
		mockFetchArticles.mockResolvedValue([]);

		render(FeedPage);
		await waitFor(() => {
			expect(screen.queryByText(/Loading/)).toBeNull();
		});

		const btn = screen.getByRole('button', { name: /새 뉴스 가져오기/i });
		fireEvent.click(btn);

		await waitFor(() => {
			expect(screen.getByText(/Network error/)).toBeTruthy();
		});
	});
});
