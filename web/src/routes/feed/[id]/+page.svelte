<script lang="ts">
	import { page } from '$app/state';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { fetchArticleById } from '$lib/utils/api';
	import { formatArticleDate, extractDomain } from '$lib/utils/article';
	import type { Article } from '$lib/types/article';

	const auth = getAuth();

	let article = $state<Article | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	$effect(() => {
		if (!auth.isAuthenticated) {
			goto('/login');
		}
	});

	$effect(() => {
		if (auth.isAuthenticated) {
			loadArticle();
		}
	});

	async function loadArticle() {
		loading = true;
		error = null;
		try {
			const id = page.params.id as string;
			if (!id) {
				error = 'Invalid article ID';
				return;
			}
			article = await fetchArticleById(id);
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
	<header class="border-b bg-white px-6 py-4">
		<div class="mx-auto flex max-w-4xl items-center gap-4">
			<a
				href="/feed"
				class="rounded-lg border border-gray-300 px-3 py-1.5 text-sm text-gray-700 hover:bg-gray-50"
			>
				Back
			</a>
			<h1 class="text-xl font-bold text-gray-900">Article Detail</h1>
		</div>
	</header>

	<main class="mx-auto max-w-4xl px-6 py-8">
		{#if loading}
			<div class="py-12 text-center text-gray-500">
				<p>Loading article...</p>
			</div>
		{:else if error}
			<div class="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
				{error}
			</div>
		{:else if article}
			<article class="rounded-lg border border-gray-200 bg-white p-6">
				<h2 class="text-2xl font-bold text-gray-900">{article.title}</h2>

				<div class="mt-3 flex items-center gap-4 text-sm text-gray-500">
					<span>{article.source || extractDomain(article.url)}</span>
					{#if article.published_at}
						<span>{formatArticleDate(article.published_at)}</span>
					{/if}
					<a
						href={article.url}
						target="_blank"
						rel="noopener noreferrer"
						class="text-blue-600 hover:underline"
					>
						원문 보기
					</a>
				</div>

				{#if article.snippet}
					<p class="mt-4 text-gray-600">{article.snippet}</p>
				{/if}

				<div class="mt-6 space-y-4">
					{#if article.summary}
						<div class="rounded-lg bg-gray-50 p-4">
							<h3 class="text-sm font-semibold text-gray-500">요약</h3>
							<p class="mt-1 text-gray-800">{article.summary}</p>
						</div>
					{:else}
						<div class="rounded-lg bg-gray-50 p-4">
							<h3 class="text-sm font-semibold text-gray-500">요약</h3>
							<p class="mt-1 text-sm text-gray-400 italic">요약 대기 중...</p>
						</div>
					{/if}

					{#if article.insight}
						<div class="rounded-lg bg-blue-50 p-4">
							<h3 class="text-sm font-semibold text-blue-600">인사이트</h3>
							<p class="mt-1 text-blue-800">{article.insight}</p>
						</div>
					{:else}
						<div class="rounded-lg bg-blue-50 p-4">
							<h3 class="text-sm font-semibold text-blue-600">인사이트</h3>
							<p class="mt-1 text-sm text-blue-400 italic">인사이트 대기 중...</p>
						</div>
					{/if}
				</div>

				{#if article.summarized_at}
					<p class="mt-4 text-xs text-gray-400">
						요약 생성: {formatArticleDate(article.summarized_at)}
					</p>
				{/if}
			</article>
		{/if}
	</main>
</div>
