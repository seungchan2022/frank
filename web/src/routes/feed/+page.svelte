<script lang="ts">
	import { getAuth } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import {
		collectArticles,
		fetchArticles,
		fetchMyTagIds,
		fetchTags,
		summarizeArticles
	} from '$lib/utils/api';
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
	let error = $state<string | null>(null);
	let hasMore = $state(true);
	let sentinel = $state<HTMLDivElement | null>(null);
	let selectedTagId = $state<string | null>(null);

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

		return () => observer.disconnect();
	});

	async function loadInitial() {
		loading = true;
		error = null;
		try {
			const [arts, allTags, tagIds] = await Promise.all([
				fetchArticles(0, 10, selectedTagId ?? undefined),
				fetchTags(),
				fetchMyTagIds()
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
			const moreArticles = await fetchArticles(
				articles.length,
				10,
				selectedTagId ?? undefined
			);
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
			await collectArticles();
			const [arts, allTags] = await Promise.all([
				fetchArticles(0, 10, selectedTagId ?? undefined),
				fetchTags()
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
		summarizing = true;
		try {
			await summarizeArticles();
			const arts = await fetchArticles(
				0,
				articles.length || 10,
				selectedTagId ?? undefined
			);
			articles = arts;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to summarize articles';
		} finally {
			summarizing = false;
		}
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
					{summarizing ? 'Summarizing...' : 'Summarize'}
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

		{#if error}
			<div class="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
				{error}
			</div>
		{:else if loading && articles.length === 0}
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
