<script lang="ts">
	import { tick, onMount, onDestroy } from 'svelte';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { feedStore } from '$lib/stores/feedStore.svelte';
	import { likedStore } from '$lib/stores/liked.svelte';
	import { formatArticleDate, extractDomain } from '$lib/utils/article';
	import type { FeedItem } from '$lib/types/article';
	import Header from '$lib/components/Header.svelte';

	const auth = getAuth();

	const tagMap = $derived(
		feedStore.tags.reduce<Record<string, string>>((acc, tag) => {
			acc[tag.id] = tag.name;
			return acc;
		}, {})
	);

	// 사용자가 구독한 태그만 필터 탭에 표시
	const filterTags = $derived(feedStore.tags.filter((tag) => feedStore.myTagIds.includes(tag.id)));

	// 인증 상태 확정 후 리다이렉트 or 초기 로드
	$effect(() => {
		if (auth.loading) return;
		if (!auth.isAuthenticated) {
			goto('/login');
			return;
		}
		feedStore.loadFeed(auth.user?.id);
	});

	function selectTag(tagId: string | null) {
		feedStore.selectTag(tagId);
	}


	// refresh 성공 시 상단 스크롤 (stale-while-revalidate UX)
	async function handleRefresh() {
		const ok = await feedStore.refresh();
		if (ok && feedStore.feedItems.length > 0) {
			await tick();
			window.scrollTo({ top: 0, behavior: 'smooth' });
		}
	}

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

	// MVP12 M2: 무한 스크롤 IntersectionObserver
	let sentinel: HTMLDivElement | undefined = $state();
	let observer: IntersectionObserver | undefined;

	onMount(() => {
		observer = new IntersectionObserver(
			(entries) => {
				const entry = entries[0];
				if (entry?.isIntersecting && feedStore.hasMore && !feedStore.isLoadingMore) {
					void feedStore.loadMore();
				}
			},
			{ rootMargin: '200px' }
		);
	});

	// sentinel이 DOM에 바인딩될 때(feedItems 첫 렌더 후) observer 연결.
	// onMount 시점에는 sentinel이 없으므로 $effect로 처리.
	// 주의: cleanup 시점에 sentinel은 이미 undefined일 수 있으므로 로컬 캡처 필수.
	$effect(() => {
		const el = sentinel;
		if (observer && el) {
			observer.observe(el);
			return () => observer?.unobserve(el);
		}
	});

	onDestroy(() => {
		observer?.disconnect();
	});
</script>

<div class="min-h-screen bg-gray-50">
	<Header />

	<main class="mx-auto max-w-4xl px-6 py-8">
		<div class="mb-6 flex items-center justify-between">
			<h2 class="text-lg font-semibold text-gray-900">My Feed</h2>
			<button
				onclick={handleRefresh}
				disabled={feedStore.loading || feedStore.isRefreshing}
				class="flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
			>
				{#if feedStore.isRefreshing}
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

		<!-- Progress Bar: refresh 중 카드 목록 위 표시 -->
		{#if feedStore.isRefreshing}
			<div class="mb-2 h-1 w-full animate-pulse rounded bg-blue-500"></div>
		{/if}

		<!-- 태그 필터 탭 -->
		<div class="mb-4 flex flex-wrap gap-2">
			<button
				onclick={() => selectTag(null)}
				class="rounded-full px-3 py-1 text-sm font-medium transition-colors {feedStore.activeTagId === null
					? 'bg-gray-900 text-white'
					: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
			>
				전체
			</button>
			{#each filterTags as tag (tag.id)}
				<button
					onclick={() => selectTag(tag.id)}
					class="rounded-full px-3 py-1 text-sm font-medium transition-colors {feedStore.activeTagId ===
					tag.id
						? 'bg-blue-600 text-white'
						: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
				>
					{tag.name}
				</button>
			{/each}
		</div>

		<!-- 에러 배너 -->
		{#if feedStore.error}
			<div class="mb-4 rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
				{feedStore.error}
			</div>
		{/if}

		{#if feedStore.loading && feedStore.feedItems.length === 0}
			<div class="py-12 text-center text-gray-500">
				<p>Loading feed...</p>
			</div>
		{:else if feedStore.feedItems.length === 0}
			<div class="py-12 text-center text-gray-500">
				<p class="text-lg">No articles yet.</p>
				<p class="mt-2 text-sm">새 뉴스 가져오기를 눌러 피드를 불러오세요.</p>
			</div>
		{:else}
			<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
				{#each feedStore.feedItems as item (item.url)}
					<article class="flex rounded-lg border border-gray-200 bg-white p-4">
						<!-- 썸네일 영역 (72×72) -->
						<div class="mr-3 h-18 w-18 flex-shrink-0">
							{#if item.image_url}
								<img
									src={item.image_url}
									alt=""
									class="h-18 w-18 rounded-lg object-cover"
									onerror={(e) => {
										const el = e.currentTarget as HTMLImageElement;
										el.style.display = 'none';
										el.nextElementSibling?.classList.remove('hidden');
									}}
								/>
								<div class="hidden h-18 w-18 rounded-lg bg-gray-200"></div>
							{:else}
								<div class="h-18 w-18 rounded-lg bg-gray-200"></div>
							{/if}
						</div>

						<!-- 텍스트 영역 -->
						<div class="flex min-w-0 flex-1 flex-col">
							<div class="mb-1 flex items-center gap-2">
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

							<button
								onclick={() => navigateToArticle(item)}
								class="line-clamp-2 text-left text-sm font-semibold text-gray-900 hover:text-blue-600"
							>
								{item.title}
							</button>

							<div class="mt-auto flex items-center justify-between pt-2">
							<div class="text-xs text-gray-400">
								{#if item.published_at}
									<span>{formatArticleDate(item.published_at)}</span>
								{/if}
							</div>
							<!-- 좋아요 버튼 (MVP12 M2 UX 재설계: 역할 명확화) -->
							<button
								onclick={(e) => {
									e.stopPropagation();
									likedStore.likeArticle({
										url: item.url,
										title: item.title,
										snippet: item.snippet ?? null,
										tag_id: item.tag_id ?? null
									});
								}}
								aria-label={likedStore.isLiked(item.url) ? '추천 완료' : '추천에 반영'}
								title="추천에 반영"
								class="flex items-center gap-0.5 text-xs transition-colors {likedStore.isLiked(item.url)
									? 'text-red-500'
									: 'text-gray-300 hover:text-red-400'}"
							>
								<span>{likedStore.isLiked(item.url) ? '♥' : '♡'}</span>
								{#if likedStore.isLiked(item.url)}
									<span class="text-red-400">추천중</span>
								{/if}
							</button>
						</div>
						</div>
					</article>
				{/each}
			</div>

			<!-- MVP12 M2: 무한 스크롤 sentinel + 상태 표시 -->
			{#if feedStore.isLoadingMore}
				<!-- loadingMore 스피너 (U-03) -->
				<div class="mt-6 flex justify-center py-4">
					<svg class="h-6 w-6 animate-spin text-blue-500" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
					</svg>
					<span class="ml-2 text-sm text-gray-500">더 불러오는 중...</span>
				</div>
			{:else if !feedStore.hasMore}
				<!-- 모든 기사 로드 완료 (U-04) -->
				<div class="mt-6 py-4 text-center text-sm text-gray-400">
					모든 기사를 읽었습니다
				</div>
			{/if}

			<!-- sentinel 엘리먼트: IntersectionObserver 감지점 -->
			<div bind:this={sentinel} class="h-1" aria-hidden="true"></div>
		{/if}
	</main>
</div>
