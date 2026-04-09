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

				<!-- Snippet section -->
				{#if article.snippet}
					<div class="rounded-lg border border-gray-200 bg-white p-6">
						<h2 class="mb-2 text-sm font-semibold tracking-wide text-gray-500 uppercase">
							원문 리드
						</h2>
						<p class="text-sm text-gray-600 leading-relaxed">{article.snippet}</p>
					</div>
				{/if}
			</article>
		{/if}
	</main>
</div>
