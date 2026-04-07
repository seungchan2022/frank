<script lang="ts">
	import { page } from '$app/state';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { apiClient } from '$lib/api';
	import { formatArticleDate, extractDomain } from '$lib/utils/article';
	import type { Article } from '$lib/types/article';
	import type { Tag } from '$lib/types/tag';
	import Header from '$lib/components/Header.svelte';

	const auth = getAuth();

	let article = $state<Article | null>(null);
	let tags = $state<Tag[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let snippetExpanded = $state(false);

	const tagMap = $derived(
		tags.reduce<Record<string, string>>((acc, tag) => {
			acc[tag.id] = tag.name;
			return acc;
		}, {})
	);

	$effect(() => {
		if (!auth.isAuthenticated) {
			goto('/login');
		}
	});

	$effect(() => {
		if (auth.isAuthenticated) {
			loadData();
		}
	});

	async function loadData() {
		loading = true;
		error = null;
		try {
			const id = page.params.id as string;
			if (!id) {
				error = 'Invalid article ID';
				return;
			}
			const [art, allTags] = await Promise.all([
				apiClient.fetchArticleById(id),
				apiClient.fetchTags()
			]);
			article = art;
			tags = allTags;
			if (!article) {
				error = 'Article not found';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load article';
		} finally {
			loading = false;
		}
	}
</script>

<div class="min-h-screen bg-gray-50">
	<Header />

	<main class="mx-auto max-w-4xl px-6 py-8">
		<div class="mb-6">
			<a href="/feed" class="text-sm text-gray-500 hover:text-gray-700">&larr; Back to Feed</a>
		</div>

		{#if loading}
			<div class="py-12 text-center text-gray-500">
				<p>Loading article...</p>
			</div>
		{:else if error}
			<div class="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
				{error}
			</div>
		{:else if article}
			<article class="space-y-6">
				<!-- Header section -->
				<div class="rounded-lg border border-gray-200 bg-white p-6">
					<div class="mb-3 flex flex-wrap items-center gap-2">
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
						{#if article.published_at}
							<span class="text-xs text-gray-400">
								{formatArticleDate(article.published_at)}
							</span>
						{/if}
					</div>

					<h1 class="text-2xl font-bold text-gray-900">{article.title}</h1>

					<div class="mt-4">
						<a
							href={article.url}
							target="_blank"
							rel="noopener noreferrer"
							class="inline-block rounded-lg border border-blue-300 bg-blue-50 px-4 py-2 text-sm font-medium text-blue-700 hover:bg-blue-100"
						>
							원문 보기
						</a>
					</div>
				</div>

				<!-- Summary section -->
				<div class="rounded-lg border border-gray-200 bg-white p-6">
					<h2 class="mb-2 text-sm font-semibold tracking-wide text-gray-500 uppercase">
						요약
					</h2>
					{#if article.summary}
						<p class="text-gray-800 leading-relaxed">{article.summary}</p>
					{:else}
						<p class="text-sm text-gray-400 italic">요약 대기 중...</p>
					{/if}
					{#if article.summarized_at}
						<p class="mt-3 text-xs text-gray-400">
							생성: {formatArticleDate(article.summarized_at)}
						</p>
					{/if}
				</div>

				<!-- Insight section -->
				<div class="rounded-lg border border-blue-100 bg-blue-50/50 p-6">
					<h2 class="mb-2 text-sm font-semibold tracking-wide text-blue-600 uppercase">
						인사이트
					</h2>
					{#if article.insight}
						<p class="text-blue-800 leading-relaxed">{article.insight}</p>
					{:else}
						<p class="text-sm text-blue-400 italic">인사이트 대기 중...</p>
					{/if}
				</div>

				<!-- Snippet section (collapsible) -->
				{#if article.snippet}
					<div class="rounded-lg border border-gray-200 bg-white p-6">
						<button
							onclick={() => (snippetExpanded = !snippetExpanded)}
							class="flex w-full items-center justify-between text-left"
						>
							<h2 class="text-sm font-semibold tracking-wide text-gray-500 uppercase">
								원문 스니펫
							</h2>
							<span class="text-xs text-gray-400">
								{snippetExpanded ? '접기' : '펼치기'}
							</span>
						</button>
						{#if snippetExpanded}
							<p class="mt-3 text-sm text-gray-600 leading-relaxed">{article.snippet}</p>
						{/if}
					</div>
				{/if}
			</article>
		{/if}
	</main>
</div>
