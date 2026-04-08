// feed/+page.svelte 요약 타임아웃 UX 테스트.
// iOS M2 FeedFeatureTests와 동일한 시나리오:
//   1. summarizingTimeout 초기값 false
//   2. 30s 타이머 발화 시 타임아웃 배너 표시
//   3. 타이머 이내 완료 시 배너 미표시
//   4. retrySummarize: summarizingTimeout 초기화 + 재요청

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
const mockSummarizeArticles = vi.fn<(signal?: AbortSignal) => Promise<number>>();
const mockFetchArticles = vi.fn();
const mockFetchTags = vi.fn();
const mockFetchMyTagIds = vi.fn();

vi.mock('$lib/api', () => ({
	apiClient: {
		summarizeArticles: (...args: [AbortSignal?]) => mockSummarizeArticles(...args),
		fetchArticles: (...args: unknown[]) => mockFetchArticles(...args),
		fetchTags: () => mockFetchTags(),
		fetchMyTagIds: () => mockFetchMyTagIds(),
		collectArticles: vi.fn().mockResolvedValue(0)
	}
}));

import FeedPage from '../+page.svelte';

beforeEach(() => {
	vi.useFakeTimers();
	vi.clearAllMocks();

	// 기본: 즉시 완료 (articles 빈 배열)
	mockFetchArticles.mockResolvedValue([]);
	mockFetchTags.mockResolvedValue([]);
	mockFetchMyTagIds.mockResolvedValue([]);
});

afterEach(() => {
	vi.useRealTimers();
	cleanup();
});

// AbortSignal이 abort될 때 AbortError로 reject하는 슬로우 promise 생성 헬퍼
function slowSummarize(signal?: AbortSignal): Promise<number> {
	return new Promise((_, reject) => {
		if (signal) {
			signal.addEventListener('abort', () => {
				reject(new DOMException('The operation was aborted.', 'AbortError'));
			});
		}
		// 신호 없으면 영원히 pending (테스트에서 fake timer로 제어)
	});
}

describe('feed/+page.svelte — 요약 타임아웃 UX (iOS M2 싱크)', () => {
	it('초기 상태: 타임아웃 배너 없음', async () => {
		render(FeedPage);

		await waitFor(() => {
			expect(screen.queryByText(/오래 걸리고 있어요/)).toBeNull();
		});
	});

	it('Summarize 버튼 클릭 시 진행 배너 표시', async () => {
		// 영원히 pending (abort될 때까지 대기)
		mockSummarizeArticles.mockImplementation(slowSummarize);

		render(FeedPage);
		await vi.runAllTimersAsync(); // loadInitial 완료

		const btn = screen.getByRole('button', { name: /Summarize/i });
		fireEvent.click(btn);

		await waitFor(() => {
			expect(screen.getByText(/AI가 요약하고 있어요/)).toBeTruthy();
		});
	});

	it('30s 타이머 발화 시 타임아웃 배너 + 재시도 버튼 표시 (iOS: isSummarizingTimeout = true)', async () => {
		mockSummarizeArticles.mockImplementation(slowSummarize);

		render(FeedPage);
		await vi.runAllTimersAsync(); // loadInitial 완료

		const btn = screen.getByRole('button', { name: /Summarize/i });
		fireEvent.click(btn);

		// 30초 경과 → AbortController.abort() → AbortError
		await vi.advanceTimersByTimeAsync(30_000);
		await vi.runAllTimersAsync();

		await waitFor(() => {
			expect(screen.getByText(/오래 걸리고 있어요/)).toBeTruthy();
			expect(screen.getByRole('button', { name: /다시 시도/i })).toBeTruthy();
		});
	});

	it('타임아웃 이내 완료 시 배너 미표시 (iOS: isSummarizingTimeout = false 유지)', async () => {
		// 즉시 성공 (기본 mock)
		mockSummarizeArticles.mockResolvedValue(3);

		render(FeedPage);
		await vi.runAllTimersAsync();

		const btn = screen.getByRole('button', { name: /Summarize/i });
		fireEvent.click(btn);

		await vi.runAllTimersAsync();

		await waitFor(() => {
			expect(screen.queryByText(/오래 걸리고 있어요/)).toBeNull();
			expect(screen.queryByRole('button', { name: /다시 시도/i })).toBeNull();
		});
	});

	it('재시도 버튼 클릭: summarizingTimeout 초기화 + summarizeArticles 재호출 (iOS: retrySummarize)', async () => {
		// 첫 번째 호출: 슬로우 (타임아웃 유발)
		mockSummarizeArticles.mockImplementationOnce(slowSummarize);
		// 두 번째 호출: 즉시 성공
		mockSummarizeArticles.mockResolvedValueOnce(2);

		render(FeedPage);
		await vi.runAllTimersAsync();

		// 첫 번째 요약 → 타임아웃
		const btn = screen.getByRole('button', { name: /Summarize/i });
		fireEvent.click(btn);
		await vi.advanceTimersByTimeAsync(30_000);
		await vi.runAllTimersAsync();

		await waitFor(() => {
			expect(screen.getByRole('button', { name: /다시 시도/i })).toBeTruthy();
		});

		// 재시도 클릭
		const retryBtn = screen.getByRole('button', { name: /다시 시도/i });
		fireEvent.click(retryBtn);
		await vi.runAllTimersAsync();

		await waitFor(() => {
			// 타임아웃 배너 사라짐
			expect(screen.queryByText(/오래 걸리고 있어요/)).toBeNull();
		});
		// summarizeArticles 총 2회 호출 (첫 요약 + 재시도)
		expect(mockSummarizeArticles).toHaveBeenCalledTimes(2);
	});
});
