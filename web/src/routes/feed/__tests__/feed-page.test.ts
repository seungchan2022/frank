// feed/+page.svelte 피드 UI 테스트 (MVP5 M1)
// ephemeral FeedItem 기반 피드 — fetchFeed() 호출, article.id 없음.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

// ── IntersectionObserver mock (jsdom 미구현) ──────────────────────────────
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
const mockFetchFeed = vi.fn<() => Promise<import('$lib/types/article').FeedItem[]>>();
const mockFetchTags = vi.fn();
const mockFetchMyTagIds = vi.fn();

vi.mock('$lib/api', () => ({
	apiClient: {
		fetchFeed: () => mockFetchFeed(),
		fetchTags: () => mockFetchTags(),
		fetchMyTagIds: () => mockFetchMyTagIds()
	}
}));

import FeedPage from '../+page.svelte';

const sampleFeedItems: import('$lib/types/article').FeedItem[] = [
	{
		title: 'AI Revolution 2026',
		url: 'https://example.com/news/ai-revolution-2026',
		snippet: 'AI is changing everything.',
		source: 'TechCrunch',
		published_at: '2026-04-09T10:00:00Z',
		tag_id: null
	}
];

beforeEach(() => {
	vi.clearAllMocks();

	mockFetchFeed.mockResolvedValue([]);
	mockFetchTags.mockResolvedValue([]);
	mockFetchMyTagIds.mockResolvedValue([]);
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
		mockFetchFeed.mockResolvedValue(sampleFeedItems);

		render(FeedPage);

		await waitFor(() => {
			expect(screen.getByText('AI Revolution 2026')).toBeTruthy();
		});
	});

	it('기사 카드: source 표시', async () => {
		mockFetchFeed.mockResolvedValue(sampleFeedItems);

		render(FeedPage);

		await waitFor(() => {
			expect(screen.getByText('TechCrunch')).toBeTruthy();
		});
	});

	it('기사 카드: 요약(summary) 미표시 — 제목+source+날짜만', async () => {
		mockFetchFeed.mockResolvedValue(sampleFeedItems);

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

	it('새 뉴스 가져오기 버튼 클릭 → fetchFeed 재호출', async () => {
		mockFetchFeed.mockResolvedValue([]);

		render(FeedPage);
		// 초기 로드 완료 대기
		await waitFor(() => {
			expect(screen.queryByText(/Loading feed/)).toBeNull();
		});

		const btn = screen.getByRole('button', { name: /새 뉴스 가져오기/i });
		fireEvent.click(btn);

		await waitFor(() => {
			// 초기 1회 + 갱신 1회 = 최소 2회
			expect(mockFetchFeed).toHaveBeenCalledTimes(2);
		});
	});

	it('갱신 완료 후 새 아이템 표시', async () => {
		const newItem: import('$lib/types/article').FeedItem = {
			title: 'New Article After Refresh',
			url: 'https://example.com/news/new-article',
			snippet: null,
			source: 'mock',
			published_at: null,
			tag_id: null
		};
		mockFetchFeed
			.mockResolvedValueOnce([]) // loadFeed (초기)
			.mockResolvedValueOnce([newItem]); // handleRefresh

		render(FeedPage);
		await waitFor(() => {
			expect(screen.queryByText(/Loading feed/)).toBeNull();
		});

		const btn = screen.getByRole('button', { name: /새 뉴스 가져오기/i });
		fireEvent.click(btn);

		await waitFor(() => {
			expect(screen.getByText('New Article After Refresh')).toBeTruthy();
		});
	});

	it('갱신 실패 시 에러 메시지 표시', async () => {
		mockFetchFeed
			.mockResolvedValueOnce([]) // loadFeed (초기)
			.mockRejectedValueOnce(new Error('Network error')); // handleRefresh 실패

		render(FeedPage);
		await waitFor(() => {
			expect(screen.queryByText(/Loading feed/)).toBeNull();
		});

		const btn = screen.getByRole('button', { name: /새 뉴스 가져오기/i });
		fireEvent.click(btn);

		await waitFor(() => {
			expect(screen.getByText(/Network error/)).toBeTruthy();
		});
	});

	it('피드 아이템에 id 필드 없음 — url 기반 key', async () => {
		mockFetchFeed.mockResolvedValue(sampleFeedItems);

		render(FeedPage);

		await waitFor(() => {
			expect(screen.getByText('AI Revolution 2026')).toBeTruthy();
		});

		// 피드 아이템 링크가 외부 URL로 직접 연결됨
		const link = screen.getByRole('link', { name: /AI Revolution 2026/i });
		expect(link.getAttribute('href')).toBe('https://example.com/news/ai-revolution-2026');
		expect(link.getAttribute('target')).toBe('_blank');
	});
});
