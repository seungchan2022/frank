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
	import type { SummaryResult } from '$lib/types/summary';
	import type { FeedItem } from '$lib/types/article';
	import type { QuizQuestion } from '$lib/types/quiz';
	import type { ArticlePageState } from './+page';

	const auth = getAuth();

	let { data } = $props();
	// state.feedItem (goto state로 전달 시 snippet 포함) 우선, 없으면 URL params 폴백
	const feedItem = $derived(
		(page.state as Partial<ArticlePageState>).feedItem ?? data.fallbackItem
	);

	// 연관 기사 상태
	let relatedItems = $state<FeedItem[]>([]);

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

	// 연관 기사 로드
	async function loadRelatedArticles(item: FeedItem) {
		const params = new URLSearchParams();
		if (item.title) params.set('title', item.title);
		if (item.snippet) params.set('snippet', item.snippet);

		try {
			const res = await fetch(`/api/related?${params.toString()}`);
			if (!res.ok) return;
			const data = await res.json();
			if (Array.isArray(data)) {
				relatedItems = data as FeedItem[];
			}
		} catch {
			// 실패해도 조용히 무시 — 연관 기사는 보조 기능
		}
	}

	$effect(() => {
		if (feedItem.title || feedItem.snippet) {
			loadRelatedArticles(feedItem);
		}
	});

	function navigateToArticle(item: FeedItem) {
		const params = new URLSearchParams({
			url: item.url,
			title: item.title,
			source: item.source,
			...(item.snippet ? { snippet: item.snippet } : {}),
			...(item.published_at ? { published_at: item.published_at } : {}),
			...(item.tag_id ? { tag_id: item.tag_id } : {})
		});
		const path = `/feed/article?${params.toString()}`;
		goto(path, { state: { feedItem: JSON.parse(JSON.stringify(item)) } });
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

		<!-- 요약 섹션 -->
		<div class="rounded-lg border border-gray-200 bg-white p-6">
			<h2 class="mb-4 text-base font-semibold text-gray-800">AI 요약</h2>

			{#if phase.tag === 'idle'}
				<button
					onclick={handleSummarize}
					class="w-full rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700 active:bg-indigo-800"
				>
					✨ 요약하기
				</button>
			{:else if phase.tag === 'loading'}
				<div class="flex items-center gap-3 py-4 text-gray-500">
					<div
						class="h-5 w-5 animate-spin rounded-full border-2 border-indigo-600 border-t-transparent"
					></div>
					<span class="text-sm">요약 중... (최대 30초)</span>
				</div>
			{:else if phase.tag === 'done'}
				<div class="space-y-4">
					<div>
						<h3 class="mb-2 text-sm font-semibold text-gray-500 uppercase tracking-wide">요약</h3>
						<div class="prose prose-sm max-w-none text-gray-700">{@html marked(phase.result.summary)}</div>
					</div>
					<div>
						<h3 class="mb-2 text-sm font-semibold text-gray-500 uppercase tracking-wide">인사이트</h3>
						<div class="prose prose-sm max-w-none text-gray-600">{@html marked(phase.result.insight)}</div>
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

		<!-- 연관 기사 -->
		{#if relatedItems.length > 0}
			<div class="mt-6 rounded-lg border border-gray-200 bg-white p-6">
				<h2 class="mb-4 text-base font-semibold text-gray-800">연관 기사</h2>
				<ul class="space-y-3">
					{#each relatedItems as item (item.url)}
						<li>
							<button
								onclick={() => navigateToArticle(item)}
								class="w-full rounded-lg border border-gray-100 bg-gray-50 p-3 text-left hover:bg-gray-100 transition-colors"
							>
								<p class="text-sm font-medium text-gray-800 line-clamp-2">{item.title}</p>
								{#if item.snippet}
									<p class="mt-1 text-xs text-gray-500 line-clamp-2">{item.snippet}</p>
								{/if}
								<div class="mt-1 flex items-center gap-2">
									<span class="text-xs text-gray-400">{item.source}</span>
									{#if item.published_at}
										<span class="text-xs text-gray-400">{formatArticleDate(item.published_at)}</span>
									{/if}
								</div>
							</button>
						</li>
					{/each}
				</ul>
			</div>
		{/if}

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
							<span class="h-4 w-4 animate-spin rounded-full border-2 border-indigo-600 border-t-transparent"></span>
							퀴즈 생성 중...
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
		onClose={() => { quizQuestions = null; }}
	/>
{/if}
