<script lang="ts">
	import { getAuth, signOut } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { fetchArticles } from '$lib/utils/api';
	import { formatArticleDate, extractDomain } from '$lib/utils/article';
	import type { Article } from '$lib/types/article';

	const auth = getAuth();

	let articles = $state<Article[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);

	$effect(() => {
		if (!auth.isAuthenticated) {
			goto('/login');
		}
	});

	$effect(() => {
		if (auth.isAuthenticated) {
			loadArticles();
		}
	});

	async function loadArticles() {
		loading = true;
		error = null;
		try {
			articles = await fetchArticles();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load articles';
			articles = [];
		} finally {
			loading = false;
		}
	}

	async function handleRefresh() {
		await loadArticles();
	}

	async function handleSignOut() {
		await signOut();
		goto('/login');
	}
</script>

<div class="min-h-screen bg-gray-50">
	<header class="border-b bg-white px-6 py-4">
		<div class="mx-auto flex max-w-4xl items-center justify-between">
			<h1 class="text-xl font-bold text-gray-900">Frank</h1>
			<div class="flex items-center gap-4">
				<span class="text-sm text-gray-600">{auth.user?.email}</span>
				<button
					onclick={handleSignOut}
					class="rounded-lg border border-gray-300 px-3 py-1.5 text-sm text-gray-700 hover:bg-gray-50"
				>
					Sign Out
				</button>
			</div>
		</div>
	</header>

	<main class="mx-auto max-w-4xl px-6 py-8">
		<div class="mb-6 flex items-center justify-between">
			<h2 class="text-lg font-semibold text-gray-900">My Feed</h2>
			<button
				onclick={handleRefresh}
				disabled={loading}
				class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
			>
				{loading ? 'Loading...' : 'Refresh'}
			</button>
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
			<div class="space-y-3">
				{#each articles as article (article.id)}
					<article class="rounded-lg border border-gray-200 bg-white p-4">
						<div class="flex items-start justify-between gap-3">
							<div class="min-w-0 flex-1">
								<a
									href={article.url}
									target="_blank"
									rel="noopener noreferrer"
									class="text-base font-medium text-gray-900 hover:text-blue-600 hover:underline"
								>
									{article.title}
								</a>
								{#if article.snippet}
									<p class="mt-1 line-clamp-2 text-sm text-gray-600">
										{article.snippet}
									</p>
								{/if}
								<div class="mt-2 flex items-center gap-3 text-xs text-gray-400">
									<span>{article.source || extractDomain(article.url)}</span>
									{#if article.published_at}
										<span>{formatArticleDate(article.published_at)}</span>
									{/if}
								</div>
							</div>
						</div>
					</article>
				{/each}
			</div>
		{/if}
	</main>
</div>
