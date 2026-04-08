<script lang="ts">
	import { getAuth } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { apiClient } from '$lib/api';
	import { formatArticleDate, extractDomain } from '$lib/utils/article';
	import type { Article } from '$lib/types/article';
	import type { Tag } from '$lib/types/tag';
	import Header from '$lib/components/Header.svelte';
	import { onMount } from 'svelte';

	const auth = getAuth();

	let articles = $state<Article[]>([]);
	let tags = $state<Tag[]>([]);
	let loading = $state(false);
	let loadingMore = $state(false);
	let summarizing = $state(false);
	let summarizingTimeout = $state(false);
	let error = $state<string | null>(null);
	let hasMore = $state(true);
	let sentinel = $state<HTMLDivElement | null>(null);
	let selectedTagId = $state<string | null>(null);

	const SUMMARIZE_TIMEOUT_MS = 30_000;
	let summarizeTimerId: ReturnType<typeof setTimeout> | null = null;

	function clearSummarizeTimer() {
		if (summarizeTimerId !== null) {
			clearTimeout(summarizeTimerId);
			summarizeTimerId = null;
		}
	}

	const tagMap = $derived(
		tags.reduce<Record<string, string>>((acc, tag) => {
			acc[tag.id] = tag.name;
			return acc;
		}, {})
	);

	let myTagIds = $state<string[]>([]);

	// 사용자가 구독한 태그만 필터 탭에 표시
	const filterTags = $derived(tags.filter((tag) => myTagIds.includes(tag.id)));

	$effect(() => {
		if (!auth.isAuthenticated) {
			goto('/login');
		}
	});

	let initialLoaded = false;

	onMount(() => {
		const observer = new IntersectionObserver(
			(entries) => {
				if (entries[0].isIntersecting && hasMore && !loadingMore && !loading) {
					loadMore();
				}
			},
			{ threshold: 0.1 }
		);

		$effect(() => {
			if (!auth.loading && auth.isAuthenticated && !initialLoaded) {
				initialLoaded = true;
				loadInitial();
			}
		});

		$effect(() => {
			if (sentinel) {
				const el = sentinel;
				observer.observe(el);
				return () => observer.unobserve(el);
			}
		});

		return () => {
			observer.disconnect();
			clearSummarizeTimer(); // 언마운트 시 진행 중인 타이머 정리
		};
	});

	async function loadInitial() {
		loading = true;
		error = null;
		try {
			const [arts, allTags, tagIds] = await Promise.all([
				apiClient.fetchArticles({ offset: 0, limit: 10, tagId: selectedTagId ?? undefined }),
				apiClient.fetchTags(),
				apiClient.fetchMyTagIds()
			]);
			articles = arts;
			tags = allTags;
			myTagIds = tagIds;
			hasMore = arts.length >= 10;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load articles';
			articles = [];
		} finally {
			loading = false;
		}
	}

	async function loadMore() {
		if (loadingMore || !hasMore) return;
		loadingMore = true;
		try {
			const moreArticles = await apiClient.fetchArticles({
				offset: articles.length,
				limit: 10,
				tagId: selectedTagId ?? undefined
			});
			if (moreArticles.length === 0) {
				hasMore = false;
			} else {
				articles = [...articles, ...moreArticles];
				hasMore = moreArticles.length >= 10;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load more articles';
		} finally {
			loadingMore = false;
		}
	}

	async function selectTag(tagId: string | null) {
		selectedTagId = tagId;
		articles = [];
		hasMore = true;
		await loadInitial();
	}

	async function handleRefresh() {
		loading = true;
		error = null;
		try {
			await apiClient.collectArticles();
			const [arts, allTags] = await Promise.all([
				apiClient.fetchArticles({ offset: 0, limit: 10, tagId: selectedTagId ?? undefined }),
				apiClient.fetchTags()
			]);
			articles = arts;
			tags = allTags;
			hasMore = arts.length >= 10;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to collect articles';
		} finally {
			loading = false;
		}
	}

	async function handleSummarize() {
		const controller = new AbortController();

		// 이전 요약 상태 전부 초기화 후 시작 (부분 리셋 시 이전 에러/타임아웃 잔류 방지)
		summarizing = true;
		summarizingTimeout = false;
		error = null;

		// 30초 후 요청을 중단해 타임아웃 배너로 전환
		summarizeTimerId = setTimeout(() => {
			summarizingTimeout = true;
			controller.abort();
		}, SUMMARIZE_TIMEOUT_MS);

		try {
			await apiClient.summarizeArticles(controller.signal);
			const arts = await apiClient.fetchArticles({
				offset: 0,
				limit: articles.length || 10,
				tagId: selectedTagId ?? undefined
			});
			articles = arts;
		} catch (e) {
			// AbortError는 타임아웃 콜백에서 이미 summarizingTimeout = true로 처리됨
			if (!(e instanceof DOMException && e.name === 'AbortError')) {
				error = e instanceof Error ? e.message : 'Failed to summarize articles';
			}
		} finally {
			clearSummarizeTimer();
			summarizing = false;
		}
	}

	async function retrySummarize() {
		// 타임아웃 배너에서 "다시 시도" 클릭 시: 잔류 타이머 정리 후 재시작
		// summarizing guard: 이미 요약 중이면 중복 실행 방지
		if (summarizing) return;
		summarizingTimeout = false;
		clearSummarizeTimer();
		await handleSummarize();
	}
</script>

<div class="min-h-screen bg-gray-50">
	<Header />

	<main class="mx-auto max-w-4xl px-6 py-8">
		<div class="mb-6 flex items-center justify-between">
			<h2 class="text-lg font-semibold text-gray-900">My Feed</h2>
			<div class="flex items-center gap-2">
				<button
					onclick={handleSummarize}
					disabled={summarizing || loading}
					class="rounded-lg border border-purple-300 bg-purple-50 px-4 py-2 text-sm font-medium text-purple-700 hover:bg-purple-100 disabled:opacity-50"
				>
					Summarize
				</button>
				<button
					onclick={handleRefresh}
					disabled={loading}
					class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
				>
					{loading ? 'Loading...' : 'Refresh'}
				</button>
			</div>
		</div>

		<!-- 태그 필터 탭 -->
		<div class="mb-4 flex flex-wrap gap-2">
			<button
				onclick={() => selectTag(null)}
				class="rounded-full px-3 py-1 text-sm font-medium transition-colors {selectedTagId === null
					? 'bg-gray-900 text-white'
					: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
			>
				전체
			</button>
			{#each filterTags as tag (tag.id)}
				<button
					onclick={() => selectTag(tag.id)}
					class="rounded-full px-3 py-1 text-sm font-medium transition-colors {selectedTagId ===
					tag.id
						? 'bg-blue-600 text-white'
						: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
				>
					{tag.name}
				</button>
			{/each}
		</div>

		<!-- 요약 진행 배너 -->
		{#if summarizing && !summarizingTimeout}
			<div class="mb-4 flex items-center gap-2 rounded-lg bg-gray-100 px-4 py-3 text-sm text-gray-500">
				<svg class="h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
				</svg>
				AI가 요약하고 있어요...
			</div>
		{/if}

		<!-- 타임아웃 배너 — summarizing 여부와 무관하게 표시 -->
		{#if summarizingTimeout}
			<div class="mb-4 flex items-center gap-3 rounded-lg bg-orange-50 px-4 py-3 text-sm">
				<svg class="h-4 w-4 flex-shrink-0 text-orange-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" stroke-linejoin="round" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
				</svg>
				<span class="flex-1 text-gray-600">요약이 오래 걸리고 있어요</span>
				<button
					onclick={retrySummarize}
					class="rounded border border-orange-300 bg-white px-3 py-1 text-xs font-medium text-orange-600 hover:bg-orange-50"
				>
					다시 시도
				</button>
			</div>
		{/if}

		<!-- 에러 배너 -->
		{#if error}
			<div class="mb-4 rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
				{error}
			</div>
		{/if}

		{#if loading && articles.length === 0}
			<div class="py-12 text-center text-gray-500">
				<p>Loading articles...</p>
			</div>
		{:else if articles.length === 0}
			<div class="py-12 text-center text-gray-500">
				<p class="text-lg">No articles yet.</p>
				<p class="mt-2 text-sm">Articles will appear here once collection runs.</p>
			</div>
		{:else}
			<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
				{#each articles as article (article.id)}
					<article class="flex flex-col rounded-lg border border-gray-200 bg-white p-4">
						<div class="mb-2 flex items-center gap-2">
							<span
								class="inline-block rounded bg-gray-200 px-2 py-0.5 text-xs font-medium text-gray-600"
							>
								{article.source || extractDomain(article.url)}
							</span>
							{#if article.tag_id && tagMap[article.tag_id]}
								<span
									class="inline-block rounded bg-blue-100 px-2 py-0.5 text-xs font-medium text-blue-700"
								>
									{tagMap[article.tag_id]}
								</span>
							{/if}
						</div>

						<a
							href="/feed/{article.id}"
							class="text-base font-semibold text-gray-900 hover:text-blue-600"
						>
							{article.title}
						</a>

						{#if article.snippet}
							<p class="mt-1 line-clamp-2 text-sm text-gray-500">
								{article.snippet}
							</p>
						{/if}

						{#if article.summary}
							<p class="mt-2 line-clamp-1 text-sm text-gray-600 italic">
								{article.summary}
							</p>
						{:else}
							<p class="mt-2 text-xs text-gray-400 italic">요약 대기 중...</p>
						{/if}

						<div class="mt-auto flex items-center gap-3 pt-3 text-xs text-gray-400">
							{#if article.published_at}
								<span>{formatArticleDate(article.published_at)}</span>
							{/if}
						</div>
					</article>
				{/each}
			</div>

			<!-- 무한 스크롤 감지 영역 -->
			<div bind:this={sentinel} class="py-8 text-center">
				{#if loadingMore}
					<p class="text-sm text-gray-400">Loading more...</p>
				{:else if !hasMore}
					<p class="text-sm text-gray-400">모든 기사를 불러왔습니다.</p>
				{/if}
			</div>
		{/if}
	</main>
</div>
