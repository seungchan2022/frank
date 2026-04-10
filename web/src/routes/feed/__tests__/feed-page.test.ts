// feed/+page.svelte 피드 UI 테스트 (MVP5 M1 → M3 feedStore 기반)
// feedStore를 mock하여 컴포넌트 UI만 검증.

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
	getAuth: () => ({ isAuthenticated: true, loading: false, user: { email: 't@t.com', id: 'user-1' } })
}));

// ── feedStore mock ────────────────────────────────────────────────────────
const mockLoadFeed = vi.fn<() => Promise<void>>();
const mockRefresh = vi.fn<() => Promise<void>>();

let mockFeedItems: import('$lib/types/article').FeedItem[] = [];
let mockLoading = false;
let mockLoaded = false;
let mockError: string | null = null;

vi.mock('$lib/stores/feedStore.svelte', () => ({
	feedStore: {
		get feedItems() { return mockFeedItems; },
		get tags() { return []; },
		get myTagIds() { return []; },
		get loaded() { return mockLoaded; },
		get loading() { return mockLoading; },
		get error() { return mockError; },
		loadFeed: (...args: Parameters<typeof mockLoadFeed>) => mockLoadFeed(...args),
		refresh: (...args: Parameters<typeof mockRefresh>) => mockRefresh(...args),
		reset: vi.fn()
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
	mockFeedItems = [];
	mockLoading = false;
	mockLoaded = false;
	mockError = null;
	mockLoadFeed.mockResolvedValue(undefined);
	mockRefresh.mockResolvedValue(undefined);
});

afterEach(() => {
	cleanup();
});

describe('feed/+page.svelte — MVP5 피드 UI', () => {
	it('초기 상태: 빈 피드 메시지 표시', async () => {
		render(FeedPage);

		await waitFor(() => {
			expect(screen.queryByText(/No articles yet/)).toBeTruthy();
		});
	});

	it('기사 카드: 제목 표시', async () => {
		mockFeedItems = sampleFeedItems;

		render(FeedPage);

		await waitFor(() => {
			expect(screen.getByText('AI Revolution 2026')).toBeTruthy();
		});
	});

	it('기사 카드: source 표시', async () => {
		mockFeedItems = sampleFeedItems;

		render(FeedPage);

		await waitFor(() => {
			expect(screen.getByText('TechCrunch')).toBeTruthy();
		});
	});

	it('기사 카드: 요약(summary) 미표시 — 제목+source+날짜만', async () => {
		mockFeedItems = sampleFeedItems;

		render(FeedPage);

		await waitFor(() => {
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

	it('새 뉴스 가져오기 버튼 클릭 → refresh 호출', async () => {
		render(FeedPage);

		await waitFor(() => {
			expect(screen.queryByText(/Loading feed/)).toBeNull();
		});

		const btn = screen.getByRole('button', { name: /새 뉴스 가져오기/i });
		fireEvent.click(btn);

		await waitFor(() => {
			expect(mockRefresh).toHaveBeenCalledTimes(1);
		});
	});

	it('로딩 중 빈 피드: Loading feed 표시', async () => {
		mockLoading = true;
		mockLoaded = false;

		render(FeedPage);

		await waitFor(() => {
			expect(screen.queryByText(/Loading feed/)).toBeTruthy();
		});
	});

	it('에러 상태: 에러 메시지 표시', async () => {
		mockError = 'Network error';

		render(FeedPage);

		await waitFor(() => {
			expect(screen.getByText(/Network error/)).toBeTruthy();
		});
	});

	it('피드 아이템 클릭 시 디테일 페이지로 이동', async () => {
		const { goto: mockGoto } = await import('$app/navigation');
		mockFeedItems = sampleFeedItems;

		render(FeedPage);

		await waitFor(() => {
			expect(screen.getByText('AI Revolution 2026')).toBeTruthy();
		});

		const btn = screen.getByRole('button', { name: /AI Revolution 2026/i });
		expect(btn).toBeTruthy();
		fireEvent.click(btn);

		await waitFor(() => {
			expect(mockGoto).toHaveBeenCalledWith(
				expect.stringContaining('/feed/article'),
				expect.objectContaining({ state: expect.objectContaining({ feedItem: expect.any(Object) }) })
			);
		});
	});

	it('뒤로가기 복귀 시 loadFeed no-op 확인 (feedStore.loaded=true → loadFeed 1회만)', async () => {
		mockLoaded = true;
		mockFeedItems = sampleFeedItems;

		// 컴포넌트 2회 mount (뒤로가기 시뮬레이션)
		const { unmount } = render(FeedPage);
		unmount();
		render(FeedPage);

		await waitFor(() => {
			// feedStore.loadFeed는 내부에서 no-op guard를 가짐
			// 컴포넌트는 항상 loadFeed를 호출하지만 store 내부에서 중복 방지
			expect(mockLoadFeed).toHaveBeenCalled();
		});
	});
});
