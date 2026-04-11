<script lang="ts">
	import { getAuth } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { apiClient } from '$lib/api';
	import { summaryCache } from '$lib/stores/summaryCache.svelte';
	import { favoritesStore } from '$lib/stores/favoritesStore.svelte';
	import { formatArticleDate } from '$lib/utils/article';
	import Header from '$lib/components/Header.svelte';
	import { marked } from 'marked';
	import type { SummaryResult } from '$lib/types/summary';
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
			<div class="mb-3 flex flex-wrap items-center gap-2">
				<span class="inline-block rounded bg-gray-200 px-2 py-0.5 text-xs font-medium text-gray-600">
					{feedItem.source}
				</span>
				{#if feedItem.published_at}
					<span class="text-xs text-gray-400">
						{formatArticleDate(feedItem.published_at)}
					</span>
				{/if}
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
	</main>
</div>
