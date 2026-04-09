<script lang="ts">
	import { getAuth } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { apiClient } from '$lib/api';
	import { formatArticleDate, extractDomain } from '$lib/utils/article';
	import type { FeedItem } from '$lib/types/article';
	import type { Tag } from '$lib/types/tag';
	import Header from '$lib/components/Header.svelte';

	const auth = getAuth();

	let feedItems = $state<FeedItem[]>([]);
	let tags = $state<Tag[]>([]);
	let loading = $state(false);
	let refreshing = $state(false);
	let error = $state<string | null>(null);
	let selectedTagId = $state<string | null>(null);
	let myTagIds = $state<string[]>([]);

	const tagMap = $derived(
		tags.reduce<Record<string, string>>((acc, tag) => {
			acc[tag.id] = tag.name;
			return acc;
		}, {})
	);

	// 사용자가 구독한 태그만 필터 탭에 표시
	const filterTags = $derived(tags.filter((tag) => myTagIds.includes(tag.id)));

	// 선택된 태그로 필터링 (tag_id 기반)
	const filteredItems = $derived(
		selectedTagId ? feedItems.filter((item) => item.tag_id === selectedTagId) : feedItems
	);

	let initialLoaded = false;

	// 인증 상태 확정 후 리다이렉트 or 초기 로드
	$effect(() => {
		if (auth.loading) return;
		if (!auth.isAuthenticated) {
			goto('/login');
			return;
		}
		if (!initialLoaded) {
			initialLoaded = true;
			loadFeed();
		}
	});

	async function loadFeed() {
		loading = true;
		error = null;
		try {
			const [items, allTags, tagIds] = await Promise.all([
				apiClient.fetchFeed(),
				apiClient.fetchTags(),
				apiClient.fetchMyTagIds()
			]);
			feedItems = items;
			tags = allTags;
			myTagIds = tagIds;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load feed';
			feedItems = [];
		} finally {
			loading = false;
		}
	}

	async function handleRefresh() {
		if (refreshing) return;
		refreshing = true;
		error = null;
		try {
			feedItems = await apiClient.fetchFeed();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to refresh feed';
		} finally {
			refreshing = false;
		}
	}

	function selectTag(tagId: string | null) {
		selectedTagId = tagId;
	}
</script>

<div class="min-h-screen bg-gray-50">
	<Header />

	<main class="mx-auto max-w-4xl px-6 py-8">
		<div class="mb-6 flex items-center justify-between">
			<h2 class="text-lg font-semibold text-gray-900">My Feed</h2>
			<button
				onclick={handleRefresh}
				disabled={refreshing || loading}
				class="flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
			>
				{#if refreshing}
					<svg class="h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
					</svg>
					갱신 중...
				{:else}
					새 뉴스 가져오기
				{/if}
			</button>
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

		<!-- 에러 배너 -->
		{#if error}
			<div class="mb-4 rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
				{error}
			</div>
		{/if}

		{#if loading && feedItems.length === 0}
			<div class="py-12 text-center text-gray-500">
				<p>Loading feed...</p>
			</div>
		{:else if filteredItems.length === 0}
			<div class="py-12 text-center text-gray-500">
				<p class="text-lg">No articles yet.</p>
				<p class="mt-2 text-sm">새 뉴스 가져오기를 눌러 피드를 불러오세요.</p>
			</div>
		{:else}
			<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
				{#each filteredItems as item (item.url)}
					<article class="flex flex-col rounded-lg border border-gray-200 bg-white p-4">
						<div class="mb-2 flex items-center gap-2">
							<span
								class="inline-block rounded bg-gray-200 px-2 py-0.5 text-xs font-medium text-gray-600"
							>
								{item.source || extractDomain(item.url)}
							</span>
							{#if item.tag_id && tagMap[item.tag_id]}
								<span
									class="inline-block rounded bg-blue-100 px-2 py-0.5 text-xs font-medium text-blue-700"
								>
									{tagMap[item.tag_id]}
								</span>
							{/if}
						</div>

						<a
							href={item.url}
							target="_blank"
							rel="noopener noreferrer"
							class="text-base font-semibold text-gray-900 hover:text-blue-600"
						>
							{item.title}
						</a>

						{#if item.snippet}
							<p class="mt-1 line-clamp-2 text-sm text-gray-500">{item.snippet}</p>
						{/if}

						<div class="mt-auto pt-3 text-xs text-gray-400">
							{#if item.published_at}
								<span>{formatArticleDate(item.published_at)}</span>
							{/if}
						</div>
					</article>
				{/each}
			</div>
		{/if}
	</main>
</div>
