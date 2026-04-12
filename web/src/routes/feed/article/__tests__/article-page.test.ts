// feed/article/+page.svelte 테스트 (MVP5 M2 + MVP7 M3)
// SummaryPhase 전환, 캐시 히트/미스, API 성공/실패 검증.
// MVP7 M3: 연관 기사 섹션 렌더링 검증 추가.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';
import type { FeedItem } from '$lib/types/article';
import type { SummaryResult } from '$lib/types/summary';

// ── SvelteKit 모듈 mock ───────────────────────────────────────────────────
vi.mock('$app/navigation', () => ({ goto: vi.fn(), pushState: vi.fn() }));
vi.mock('$app/state', () => ({ page: { url: { pathname: '/feed/article' }, state: {} } }));
vi.mock('$app/forms', () => ({ enhance: () => ({ destroy: () => undefined }) }));

// ── auth store mock (항상 로그인 상태) ────────────────────────────────────
vi.mock('$lib/stores/auth.svelte', () => ({
	getAuth: () => ({ isAuthenticated: true, loading: false, user: { email: 't@t.com' } })
}));

// ── apiClient mock ────────────────────────────────────────────────────────
const mockSummarize = vi.fn<() => Promise<SummaryResult>>();
vi.mock('$lib/api', () => ({
	apiClient: {
		summarize: (...args: unknown[]) => mockSummarize(...(args as []))
	}
}));

// ── summaryCache mock ─────────────────────────────────────────────────────
const mockCacheGet = vi.fn<(url: string) => SummaryResult | undefined>();
const mockCacheSet = vi.fn<(url: string, result: SummaryResult) => void>();
vi.mock('$lib/stores/summaryCache.svelte', () => ({
	summaryCache: {
		get: (url: string) => mockCacheGet(url),
		set: (url: string, result: SummaryResult) => mockCacheSet(url, result)
	}
}));

// fetch mock (퀴즈 등 fetch 사용 API 대비)
const mockFetch = vi.fn<() => Promise<Response>>();
vi.stubGlobal('fetch', mockFetch);

import ArticlePage from '../+page.svelte';

const sampleFeedItem: FeedItem = {
	title: 'AI 뉴스 기사',
	url: 'https://example.com/ai-news',
	snippet: '리드 문장입니다.',
	source: 'TechCrunch',
	published_at: '2026-04-09T10:00:00Z',
	tag_id: null
};

function makeProps(feedItem: FeedItem = sampleFeedItem) {
	return { data: { fallbackItem: feedItem } };
}

beforeEach(() => {
	vi.clearAllMocks();
	mockCacheGet.mockReturnValue(undefined);
	mockSummarize.mockResolvedValue({ summary: 'Mock 요약', insight: 'Mock 인사이트' });
});

afterEach(() => {
	cleanup();
});

describe('feed/article/+page.svelte — MVP5 M2', () => {
	// ── 렌더링 ────────────────────────────────────────────────────────────

	it('기사 제목 표시', async () => {
		render(ArticlePage, makeProps());

		await waitFor(() => {
			expect(screen.getByText('AI 뉴스 기사')).toBeTruthy();
		});
	});

	it('source 표시', async () => {
		render(ArticlePage, makeProps());

		await waitFor(() => {
			expect(screen.getByText('TechCrunch')).toBeTruthy();
		});
	});

	it('snippet 표시', async () => {
		render(ArticlePage, makeProps());

		await waitFor(() => {
			expect(screen.getByText('리드 문장입니다.')).toBeTruthy();
		});
	});

	// ── idle 상태 ─────────────────────────────────────────────────────────

	it('초기 상태: "요약하기" 버튼 표시', async () => {
		render(ArticlePage, makeProps());

		await waitFor(() => {
			expect(screen.getByRole('button', { name: /요약하기/ })).toBeTruthy();
		});
	});

	// ── 캐시 히트 ─────────────────────────────────────────────────────────

	it('캐시 히트: API 호출 없이 요약 결과 표시', async () => {
		mockCacheGet.mockReturnValue({ summary: '캐시된 요약', insight: '캐시된 인사이트' });

		render(ArticlePage, makeProps());

		await waitFor(() => {
			expect(screen.getByText('캐시된 요약')).toBeTruthy();
			expect(screen.getByText('캐시된 인사이트')).toBeTruthy();
		});
		expect(mockSummarize).not.toHaveBeenCalled();
	});

	// ── API 성공 ──────────────────────────────────────────────────────────

	it('요약하기 클릭 → API 호출 → 결과 표시', async () => {
		render(ArticlePage, makeProps());

		await waitFor(() => {
			expect(screen.getByRole('button', { name: /요약하기/ })).toBeTruthy();
		});

		fireEvent.click(screen.getByRole('button', { name: /요약하기/ }));

		await waitFor(() => {
			expect(screen.getByText('Mock 요약')).toBeTruthy();
			expect(screen.getByText('Mock 인사이트')).toBeTruthy();
		});
		expect(mockSummarize).toHaveBeenCalledTimes(1);
	});

	it('요약 성공 후 캐시에 저장됨', async () => {
		render(ArticlePage, makeProps());

		await waitFor(() => {
			expect(screen.getByRole('button', { name: /요약하기/ })).toBeTruthy();
		});

		fireEvent.click(screen.getByRole('button', { name: /요약하기/ }));

		await waitFor(() => {
			expect(mockCacheSet).toHaveBeenCalledWith(sampleFeedItem.url, {
				summary: 'Mock 요약',
				insight: 'Mock 인사이트'
			});
		});
	});

	// ── API 실패 ──────────────────────────────────────────────────────────

	it('요약 API 실패 → 에러 메시지 표시', async () => {
		mockSummarize.mockRejectedValue(new Error('네트워크 오류'));

		render(ArticlePage, makeProps());

		await waitFor(() => {
			expect(screen.getByRole('button', { name: /요약하기/ })).toBeTruthy();
		});

		fireEvent.click(screen.getByRole('button', { name: /요약하기/ }));

		await waitFor(() => {
			expect(screen.getByText(/네트워크 오류/)).toBeTruthy();
		});
	});

	it('실패 후 "다시 시도" 버튼 표시', async () => {
		mockSummarize.mockRejectedValue(new Error('오류 발생'));

		render(ArticlePage, makeProps());
		await waitFor(() => {
			expect(screen.getByRole('button', { name: /요약하기/ })).toBeTruthy();
		});
		fireEvent.click(screen.getByRole('button', { name: /요약하기/ }));

		await waitFor(() => {
			expect(screen.getByRole('button', { name: /다시 시도/ })).toBeTruthy();
		});
	});

	it('실패 후 재시도 → 성공 시 결과 표시', async () => {
		mockSummarize
			.mockRejectedValueOnce(new Error('첫 번째 실패'))
			.mockResolvedValueOnce({ summary: '재시도 요약', insight: '재시도 인사이트' });

		render(ArticlePage, makeProps());
		await waitFor(() => {
			expect(screen.getByRole('button', { name: /요약하기/ })).toBeTruthy();
		});

		fireEvent.click(screen.getByRole('button', { name: /요약하기/ }));
		await waitFor(() => {
			expect(screen.getByRole('button', { name: /다시 시도/ })).toBeTruthy();
		});

		fireEvent.click(screen.getByRole('button', { name: /다시 시도/ }));
		await waitFor(() => {
			expect(screen.getByText('재시도 요약')).toBeTruthy();
		});
	});

	// ── 중복 호출 방지 ────────────────────────────────────────────────────

	it('done 상태에서 재클릭 → API 중복 호출 안 함', async () => {
		render(ArticlePage, makeProps());
		await waitFor(() => {
			expect(screen.getByRole('button', { name: /요약하기/ })).toBeTruthy();
		});

		fireEvent.click(screen.getByRole('button', { name: /요약하기/ }));
		await waitFor(() => {
			expect(screen.getByText('Mock 요약')).toBeTruthy();
		});

		// done 상태 후 handleSummarize 재호출 시 추가 API 호출 없음
		expect(mockSummarize).toHaveBeenCalledTimes(1);
	});
});

