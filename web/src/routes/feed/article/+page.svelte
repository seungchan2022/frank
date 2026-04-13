<script lang="ts">
	import { getAuth } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { apiClient } from '$lib/api';
	import { summaryCache } from '$lib/stores/summaryCache.svelte';
	import { favoritesStore } from '$lib/stores/favoritesStore.svelte';
	import { likedStore } from '$lib/stores/liked.svelte';
	import { formatArticleDate } from '$lib/utils/article';
	import Header from '$lib/components/Header.svelte';
	import QuizModal from '$lib/components/QuizModal.svelte';
	import { marked } from 'marked';

	/** 마크다운 텍스트 전처리 — 단일 줄바꿈을 이중 줄바꿈으로 변환해 문단 간격 확보. */
	function renderMarkdown(text: string): string {
		const processed = text.replace(/([^\n])\n([^\n])/g, '$1\n\n$2');
		return marked(processed) as string;
	}
	import type { SummaryResult } from '$lib/types/summary';
	import type { QuizQuestion } from '$lib/types/quiz';
	import type { ArticlePageState } from './+page';

	const auth = getAuth();

	let { data } = $props();
	// state.feedItem (goto state로 전달 시 snippet 포함) 우선, 없으면 URL params 폴백
	const feedItem = $derived(
		(page.state as Partial<ArticlePageState>).feedItem ?? data.fallbackItem
	);

	// 요약 상태: idle | loading | done | failed
	type SummaryPhase =
		| { tag: 'idle' }
		| { tag: 'loading' }
		| { tag: 'done'; result: SummaryResult }
		| { tag: 'failed'; message: string };

	let phase = $state<SummaryPhase>({ tag: 'idle' });
	let favoriteLoading = $state(false);

	// 퀴즈 상태
	type QuizPhase = 'idle' | 'loading' | 'error';
	let quizPhase = $state<QuizPhase>('idle');
	let quizQuestions = $state<QuizQuestion[] | null>(null);
	let quizError = $state<string | null>(null);

	// 타이머 기반 로딩 텍스트 (8s 간격 전환)
	let summarizeLoadingText = $state('요약 중…');
	let quizLoadingText = $state('퀴즈 생성 중…');

	/** 로딩 여부에 따라 텍스트를 8s 후 "마무리 중…"으로 전환. $effect 내에서 사용. */
	function makeLoadingTextEffect(
		isLoading: () => boolean,
		setter: (v: string) => void,
		initial: string,
		delayMs = 8000
	) {
		if (!isLoading()) {
			setter(initial);
			return;
		}
		const timer = setTimeout(() => setter('마무리 중…'), delayMs);
		return () => clearTimeout(timer);
	}

	$effect(() => {
		if (!auth.isAuthenticated) {
			goto('/login');
		}
	});

	// 캐시 히트 시 자동 표시
	$effect(() => {
		const cached = summaryCache.get(feedItem.url);
		if (cached) {
			phase = { tag: 'done', result: cached };
		}
	});

	// 페이지 진입 시 즐겨찾기 로드 (이미 로드됐으면 no-op)
	$effect(() => {
		if (auth.isAuthenticated) {
			favoritesStore.loadFavorites(auth.user?.id);
		}
	});

	// 요약/퀴즈 로딩 텍스트: 8s 후 "마무리 중…"으로 전환
	$effect(() =>
		makeLoadingTextEffect(
			() => phase.tag === 'loading',
			(v) => (summarizeLoadingText = v),
			'요약 중…'
		)
	);
	$effect(() =>
		makeLoadingTextEffect(
			() => quizPhase === 'loading',
			(v) => (quizLoadingText = v),
			'퀴즈 생성 중…'
		)
	);

	async function handleSummarize() {
		// 중복 호출 방지
		if (phase.tag === 'loading' || phase.tag === 'done') return;

		// 캐시 확인
		const cached = summaryCache.get(feedItem.url);
		if (cached) {
			phase = { tag: 'done', result: cached };
			return;
		}

		phase = { tag: 'loading' };
		try {
			const result = await apiClient.summarize(feedItem.url, feedItem.title);
			summaryCache.set(feedItem.url, result);
			phase = { tag: 'done', result };
		} catch (e) {
			const message =
				e instanceof Error ? e.message : '요약을 불러오지 못했습니다. 다시 시도해주세요.';
			phase = { tag: 'failed', message };
		}
	}

	async function handleQuiz() {
		if (quizPhase === 'loading') return;
		quizPhase = 'loading';
		quizError = null;
		try {
			const res = await fetch('/api/favorites/quiz', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ url: feedItem.url })
			});
			const data = await res.json();
			if (!res.ok) {
				if (res.status === 503) {
					quizError = '퀴즈 생성에 실패했습니다. 잠시 후 다시 시도해주세요.';
				} else {
					quizError = (data as { error?: string })?.error ?? '퀴즈를 불러오지 못했습니다.';
				}
				quizPhase = 'error';
				return;
			}
			quizQuestions = (data as { questions: QuizQuestion[] }).questions;
			quizPhase = 'idle';
		} catch {
			quizError = '네트워크 오류가 발생했습니다.';
			quizPhase = 'error';
		}
	}

	async function handleFavoriteToggle() {
		if (favoriteLoading) return;
		favoriteLoading = true;
		try {
			if (favoritesStore.isLiked(feedItem.url)) {
				await favoritesStore.removeFavorite(feedItem.url);
			} else {
				const summary = phase.tag === 'done' ? phase.result.summary : undefined;
				const insight = phase.tag === 'done' ? phase.result.insight : undefined;
				await favoritesStore.addFavorite(feedItem, summary, insight);
			}
		} catch (e) {
			// 에러는 무시 (UI 일관성 유지)
			console.error('favorite toggle failed', e);
		} finally {
			favoriteLoading = false;
		}
	}
</script>

<div class="min-h-screen bg-gray-50">
	<Header />

	<main class="mx-auto max-w-4xl px-6 py-8">
		<div class="mb-6">
			<a href="/feed" class="text-sm text-gray-500 hover:text-gray-700">&larr; 피드로 돌아가기</a>
		</div>

		<!-- 기사 헤더 -->
		<div class="mb-6 rounded-lg border border-gray-200 bg-white p-6">
			<div class="mb-3 flex flex-wrap items-center justify-between gap-2">
				<div class="flex flex-wrap items-center gap-2">
					<span class="inline-block rounded bg-gray-200 px-2 py-0.5 text-xs font-medium text-gray-600">
						{feedItem.source}
					</span>
					{#if feedItem.published_at}
						<span class="text-xs text-gray-400">
							{formatArticleDate(feedItem.published_at)}
						</span>
					{/if}
				</div>
				<!-- 좋아요 하트 버튼 -->
				<button
					onclick={() => {
						likedStore.likeArticle({
							url: feedItem.url,
							title: feedItem.title,
							snippet: feedItem.snippet ?? null
						});
					}}
					aria-label={likedStore.isLiked(feedItem.url) ? '좋아요 완료' : '좋아요'}
					class="text-xl transition-colors {likedStore.isLiked(feedItem.url)
						? 'text-red-500'
						: 'text-gray-300 hover:text-red-400'}"
				>
					{likedStore.isLiked(feedItem.url) ? '♥' : '♡'}
				</button>
			</div>

			<h1 class="text-2xl font-bold text-gray-900">{feedItem.title}</h1>

			<div class="mt-4">
				<a
					href={feedItem.url}
					target="_blank"
					rel="noopener noreferrer"
					class="inline-block rounded-lg border border-blue-300 bg-blue-50 px-4 py-2 text-sm font-medium text-blue-700 hover:bg-blue-100"
				>
					원문 보기
				</a>
			</div>
		</div>

		<!-- 기사 소개 -->
		{#if feedItem.snippet}
			<div class="mb-6 rounded-lg border border-gray-200 bg-white p-6">
				<h2 class="mb-2 text-sm font-semibold tracking-wide text-gray-500 uppercase">기사 소개</h2>
				<p class="text-sm leading-relaxed text-gray-600">{feedItem.snippet}</p>
			</div>
		{/if}

		<!-- AI 요약 및 인사이트 섹션 -->
		<div class="rounded-lg border border-gray-200 bg-white p-6">
			<h2 class="mb-4 text-base font-semibold text-gray-800">AI 요약 및 인사이트</h2>

			{#if phase.tag === 'idle'}
				<button
					onclick={handleSummarize}
					class="w-full rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700 active:bg-indigo-800"
				>
					✨ 요약하기
				</button>
			{:else if phase.tag === 'loading'}
				<div class="flex items-center gap-3 py-4">
					<div
						class={[
							'h-5 w-5 animate-spin rounded-full border-2 border-t-transparent',
							summarizeLoadingText === '마무리 중…' ? 'border-orange-500' : 'border-indigo-600'
						].join(' ')}
					></div>
					<span class={[
						'text-sm',
						summarizeLoadingText === '마무리 중…' ? 'text-orange-500' : 'text-indigo-600'
					].join(' ')}>{summarizeLoadingText}</span>
				</div>
			{:else if phase.tag === 'done'}
				<div class="space-y-6">
					<div>
						<h3 class="mb-3 text-xs font-semibold tracking-widest text-gray-400 uppercase">요약</h3>
						<div class="prose prose-base max-w-none leading-relaxed text-gray-700 [&_p]:mb-4 [&_p:last-child]:mb-0">{@html renderMarkdown(phase.result.summary)}</div>
					</div>
					<div class="border-t border-gray-100 pt-6">
						<h3 class="mb-3 text-xs font-semibold tracking-widest text-gray-400 uppercase">인사이트</h3>
						<div class="prose prose-base max-w-none leading-relaxed text-gray-600 [&_p]:mb-4 [&_p:last-child]:mb-0">{@html renderMarkdown(phase.result.insight)}</div>
					</div>
				</div>
			{:else if phase.tag === 'failed'}
				<div class="space-y-3">
					<p class="text-sm text-red-600">{phase.message}</p>
					<button
						onclick={handleSummarize}
						class="rounded-lg border border-orange-300 bg-orange-50 px-4 py-2 text-sm font-medium text-orange-700 hover:bg-orange-100"
					>
						↺ 다시 시도
					</button>
				</div>
			{/if}
		</div>

		<!-- 즐겨찾기 -->
		<div class="mt-4">
			<button
				onclick={handleFavoriteToggle}
				disabled={favoriteLoading}
				class={[
					'w-full rounded-lg px-4 py-2 text-sm font-medium transition-colors',
					favoritesStore.isLiked(feedItem.url)
						? 'border border-yellow-400 bg-yellow-50 text-yellow-700 hover:bg-yellow-100'
						: 'border border-gray-200 bg-white text-gray-700 hover:bg-gray-50',
					favoriteLoading ? 'cursor-not-allowed opacity-50' : ''
				].join(' ')}
			>
				{#if favoriteLoading}
					⏳ 처리 중...
				{:else if favoritesStore.isLiked(feedItem.url)}
					★ 즐겨찾기 해제
				{:else}
					☆ 즐겨찾기 추가
				{/if}
			</button>
		</div>

		<!-- 퀴즈 버튼 (즐겨찾기한 기사만 표시) -->
		{#if favoritesStore.isLiked(feedItem.url)}
			<div class="mt-3">
				<button
					onclick={handleQuiz}
					disabled={quizPhase === 'loading'}
					class={[
						'w-full rounded-lg border border-indigo-300 bg-indigo-50 px-4 py-2 text-sm font-medium text-indigo-700 transition-colors',
						quizPhase === 'loading'
							? 'cursor-not-allowed opacity-50'
							: 'hover:bg-indigo-100'
					].join(' ')}
				>
					{#if quizPhase === 'loading'}
						<span class="flex items-center justify-center gap-2">
							<span class={[
								'h-4 w-4 animate-spin rounded-full border-2 border-t-transparent',
								quizLoadingText === '마무리 중…' ? 'border-orange-500' : 'border-indigo-600'
							].join(' ')}></span>
							<span class={quizLoadingText === '마무리 중…' ? 'text-orange-500' : ''}>{quizLoadingText}</span>
						</span>
					{:else}
						🧠 퀴즈 풀기
					{/if}
				</button>
				{#if quizPhase === 'error' && quizError}
					<p class="mt-1 text-xs text-red-600">{quizError}</p>
				{/if}
			</div>
		{/if}
	</main>
</div>

<!-- 퀴즈 모달 -->
{#if quizQuestions !== null}
	<QuizModal
		questions={quizQuestions}
		articleUrl={feedItem?.url}
		articleTitle={feedItem?.title}
		onClose={() => { quizQuestions = null; }}
	/>
{/if}
