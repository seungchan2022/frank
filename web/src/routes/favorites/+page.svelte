<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { favoritesStore } from '$lib/stores/favoritesStore.svelte';
	import { summaryCache } from '$lib/stores/summaryCache.svelte';
	import { apiClient } from '$lib/api';
	import Header from '$lib/components/Header.svelte';
	import WrongAnswerCard from '$lib/components/WrongAnswerCard.svelte';
	import { formatArticleDate } from '$lib/utils/article';
	import {
		buildFavTagIds,
		buildFilterTags,
		buildWrongAnswerTagMap,
		filterFavorites,
		filterWrongAnswers
	} from '$lib/utils/favorites-filter';
	import type { Favorite } from '$lib/types/favorite';
	import type { WrongAnswer } from '$lib/types/quiz';
	import type { Tag } from '$lib/types/tag';

	const auth = getAuth();

	/// MVP8 M3: 스크랩 탭 세그먼트 — 'articles' | 'wrong-answers'
	let activeTab = $state<'articles' | 'wrong-answers'>('articles');

	/// MVP11 M3: 태그 칩 필터
	let allTags = $state<Tag[]>([]);
	let selectedTagId = $state<string | null>(null);

	/// 오답 노트 상태 (filteredWrongAnswers 선언 전에 위치해야 함)
	let wrongAnswers = $state<WrongAnswer[]>([]);
	let wrongAnswersLoading = $state(false);
	let wrongAnswersError = $state<string | null>(null);
	let wrongAnswersLoaded = $state(false);

	/// MVP9 M2: 기사별 오답 시트 상태
	let showWrongAnswerSheet = $state(false);
	let sheetWrongAnswers = $state<WrongAnswer[]>([]);
	let sheetLoading = $state(false);
	let sheetArticleUrl = $state('');

	$effect(() => {
		if (!auth.isAuthenticated) {
			goto('/login');
		}
	});

	$effect(() => {
		if (auth.isAuthenticated) {
			favoritesStore.loadFavorites(auth.user?.id);
		}
	});

	/// 오답 노트 탭 진입 시 로드 (최초 1회)
	$effect(() => {
		if (activeTab === 'wrong-answers' && !wrongAnswersLoaded) {
			void loadWrongAnswers();
		}
	});

	async function loadWrongAnswers() {
		wrongAnswersLoading = true;
		wrongAnswersError = null;
		try {
			wrongAnswers = await apiClient.listWrongAnswers();
			wrongAnswersLoaded = true;
		} catch {
			wrongAnswersError = '오답 노트를 불러오지 못했습니다.';
		} finally {
			wrongAnswersLoading = false;
		}
	}

	async function handleDeleteWrongAnswer(id: string) {
		try {
			await apiClient.deleteWrongAnswer(id);
			wrongAnswers = wrongAnswers.filter((wa) => wa.id !== id);
		} catch {
			// 실패 시 조용히 무시 (선택적으로 에러 표시 가능)
		}
	}

	function goToArticle(fav: Favorite) {
		// 요약이 있으면 summaryCache에 미리 주입 → 디테일 페이지에서 즉시 표시
		if (fav.summary && fav.insight) {
			summaryCache.set(fav.url, { summary: fav.summary, insight: fav.insight });
		}
		const params = new URLSearchParams({
			url: fav.url,
			title: fav.title,
			source: fav.source,
			...(fav.snippet ? { snippet: fav.snippet } : {}),
			...(fav.publishedAt ? { published_at: fav.publishedAt } : {}),
			...(fav.tagId ? { tag_id: fav.tagId } : {})
		});
		goto(`/feed/article?${params.toString()}`, {
			state: {
				feedItem: {
					title: fav.title,
					url: fav.url,
					snippet: fav.snippet,
					source: fav.source,
					published_at: fav.publishedAt,
					tag_id: fav.tagId
				}
			}
		});
	}

	async function handleRemoveFavorite(url: string, event: MouseEvent) {
		event.stopPropagation();
		await favoritesStore.removeFavorite(url);
	}

	function handleRetry() {
		favoritesStore.loadFavorites(auth.user?.id);
	}

	/// MVP9 M2: 기사별 오답 시트 열기
	async function openWrongAnswerSheet(articleUrl: string) {
		showWrongAnswerSheet = true;
		sheetArticleUrl = articleUrl;
		if (!wrongAnswersLoaded) {
			sheetLoading = true;
			try {
				wrongAnswers = await apiClient.listWrongAnswers();
				wrongAnswersLoaded = true;
			} catch {
				// 로드 실패해도 시트는 열림 (빈 상태로 표시)
			} finally {
				sheetLoading = false;
			}
		}
		sheetWrongAnswers = wrongAnswers.filter((wa) => wa.articleUrl === articleUrl);
	}

	/// MVP9 M2: 오답 시트 닫기
	function closeWrongAnswerSheet() {
		showWrongAnswerSheet = false;
		sheetWrongAnswers = [];
		sheetArticleUrl = '';
	}

	/// MVP9 M2: 퀴즈 다시 풀기 — 기사 상세 페이지로 이동 (퀴즈 모달이 거기서 열림)
	function retryQuiz(fav: Favorite, event: MouseEvent) {
		event.stopPropagation();
		goToArticle(fav);
	}

	const favTagIds = $derived(buildFavTagIds(favoritesStore.favorites));
	const filterTags = $derived(buildFilterTags(allTags, favTagIds));
	const filteredFavorites = $derived(filterFavorites(favoritesStore.favorites, selectedTagId));
	const wrongAnswerTagMap = $derived(buildWrongAnswerTagMap(favoritesStore.favorites));
	const filteredWrongAnswers = $derived(filterWrongAnswers(wrongAnswers, wrongAnswerTagMap, selectedTagId));

	onMount(async () => {
		try {
			allTags = await apiClient.fetchTags();
		} catch {
			// 태그 로드 실패 시 필터 UI 숨김, 전체 목록 표시 (비크리티컬)
		}
	});
</script>

<div class="min-h-screen bg-gray-50">
	<Header />

	<main class="mx-auto max-w-4xl px-6 py-8">
		<div class="mb-6 flex items-center justify-between">
			<h1 class="text-2xl font-bold text-gray-900">스크랩</h1>
			<a href="/feed" class="text-sm text-gray-500 hover:text-gray-700">← 피드로</a>
		</div>

		<!-- 세그먼트 컨트롤 -->
		<div class="mb-6 flex rounded-lg border border-gray-200 bg-white p-1 w-fit gap-1">
			<button
				onclick={() => { activeTab = 'articles'; selectedTagId = null; }}
				class={[
					'rounded-md px-4 py-1.5 text-sm font-medium transition-colors',
					activeTab === 'articles'
						? 'bg-indigo-600 text-white'
						: 'text-gray-500 hover:text-gray-700'
				].join(' ')}
			>
				기사
			</button>
			<button
				onclick={() => { activeTab = 'wrong-answers'; selectedTagId = null; }}
				class={[
					'rounded-md px-4 py-1.5 text-sm font-medium transition-colors',
					activeTab === 'wrong-answers'
						? 'bg-indigo-600 text-white'
						: 'text-gray-500 hover:text-gray-700'
				].join(' ')}
			>
				오답 노트
			</button>
		</div>

		{#if activeTab === 'articles'}
			<!-- 태그 칩 필터 (MVP11 M3) -->
			{#if filterTags.length > 0}
				<div class="mb-4 flex flex-wrap gap-2">
					<button
						onclick={() => (selectedTagId = null)}
						class="rounded-full px-3 py-1 text-sm font-medium transition-colors {selectedTagId === null
							? 'bg-gray-900 text-white'
							: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
					>
						전체
					</button>
					{#each filterTags as tag (tag.id)}
						<button
							onclick={() => (selectedTagId = tag.id)}
							class="rounded-full px-3 py-1 text-sm font-medium transition-colors {selectedTagId === tag.id
								? 'bg-indigo-600 text-white'
								: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
						>
							{tag.name}
						</button>
					{/each}
				</div>
			{/if}

			<!-- 기사 탭 -->
			{#if favoritesStore.loading}
				<!-- 로딩 스켈레톤 -->
				<div class="space-y-3">
					{#each [1, 2, 3] as i (i)}
						<div class="animate-pulse rounded-lg border border-gray-200 bg-white p-5">
							<div class="mb-2 h-5 w-3/4 rounded bg-gray-200"></div>
							<div class="h-4 w-1/3 rounded bg-gray-100"></div>
						</div>
					{/each}
				</div>
			{:else if favoritesStore.error}
				<!-- 에러 -->
				<div class="rounded-lg border border-red-200 bg-red-50 p-6 text-center">
					<p class="mb-4 text-sm text-red-600">{favoritesStore.error}</p>
					<button
						onclick={handleRetry}
						class="rounded-lg border border-red-300 bg-white px-4 py-2 text-sm font-medium text-red-700 hover:bg-red-50"
					>
						↺ 다시 시도
					</button>
				</div>
			{:else if favoritesStore.favorites.length === 0}
				<!-- 빈 상태 -->
				<div class="rounded-lg border border-gray-200 bg-white p-12 text-center">
					<p class="text-4xl">☆</p>
					<p class="mt-4 text-base font-medium text-gray-700">즐겨찾기한 기사가 없습니다</p>
					<p class="mt-2 text-sm text-gray-500">피드에서 기사를 읽고 즐겨찾기를 추가해보세요.</p>
					<a
						href="/feed"
						class="mt-6 inline-block rounded-lg bg-indigo-600 px-5 py-2 text-sm font-medium text-white hover:bg-indigo-700"
					>
						피드 보러 가기
					</a>
				</div>
			{:else if filteredFavorites.length === 0}
				<!-- 태그 필터 결과 없음 -->
				<div class="rounded-lg border border-gray-200 bg-white p-8 text-center">
					<p class="text-sm text-gray-500">해당 태그의 즐겨찾기 기사가 없습니다.</p>
				</div>
			{:else}
				<!-- 기사 목록 -->
				<div class="space-y-3">
					{#each filteredFavorites as fav (fav.id)}
						<div class="group rounded-lg border border-gray-200 bg-white transition-shadow hover:shadow-md">
							<!-- 상단: 기사 정보 + 즐겨찾기 해제 -->
							<div class="flex items-stretch">
								<button
									onclick={() => goToArticle(fav)}
									class="min-w-0 flex-1 p-4 text-left"
								>
									<div class="flex items-start gap-3">
										<!-- 썸네일 영역 (72×72) -->
										<div class="h-18 w-18 flex-shrink-0">
											{#if fav.imageUrl}
												<img
													src={fav.imageUrl}
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
										<div class="min-w-0 flex-1">
											<div class="mb-1 flex items-start gap-2">
												<h2 class="line-clamp-2 text-sm font-semibold text-gray-900 group-hover:text-indigo-700 flex-1">
													{fav.title}
												</h2>
												{#if fav.quizCompleted}
													<!-- 퀴즈 완료 배지 -->
													<span
														class="flex-shrink-0 rounded-full bg-indigo-100 px-2 py-0.5 text-xs font-medium text-indigo-700"
														title="퀴즈 완료"
													>
														퀴즈 ✓
													</span>
												{/if}
											</div>
											<div class="flex flex-wrap items-center gap-2 text-xs text-gray-400">
												<span class="inline-block rounded bg-gray-100 px-2 py-0.5 font-medium text-gray-600">
													{fav.source}
												</span>
												{#if fav.createdAt}
													<span>스크랩: {formatArticleDate(fav.createdAt)}</span>
												{/if}
												{#if fav.summary}
													<span class="text-indigo-500">✨ 요약 있음</span>
												{/if}
											</div>
										</div>
									</div>
								</button>
								<button
									onclick={(e) => handleRemoveFavorite(fav.url, e)}
									class="flex-shrink-0 px-4 text-yellow-400 transition-colors hover:bg-red-50 hover:text-red-500"
									title="즐겨찾기 해제"
								>
									★
								</button>
							</div>

							<!-- 하단: MVP9 M2 퀴즈 버튼 재설계 -->
							{#if fav.quizCompleted}
								<div class="flex gap-2 border-t border-gray-100 px-4 py-2">
									<button
										onclick={(e) => retryQuiz(fav, e)}
										class="rounded-md border border-indigo-200 bg-indigo-50 px-3 py-1.5 text-xs font-medium text-indigo-700 transition-colors hover:bg-indigo-100"
									>
										↺ 다시 풀기
									</button>
									<button
										onclick={(e) => { e.stopPropagation(); void openWrongAnswerSheet(fav.url); }}
										class="rounded-md border border-gray-200 bg-gray-50 px-3 py-1.5 text-xs font-medium text-gray-600 transition-colors hover:bg-gray-100"
									>
										오답 보기
									</button>
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		{:else}
			<!-- 오답 노트 탭 -->
			{#if wrongAnswersLoading}
				<div class="space-y-3">
					{#each [1, 2, 3] as i (i)}
						<div class="animate-pulse rounded-lg border border-gray-200 bg-white p-5">
							<div class="mb-2 h-4 w-2/3 rounded bg-gray-200"></div>
							<div class="mb-3 h-5 w-full rounded bg-gray-200"></div>
							<div class="h-4 w-1/2 rounded bg-gray-100"></div>
						</div>
					{/each}
				</div>
			{:else if wrongAnswersError}
				<div class="rounded-lg border border-red-200 bg-red-50 p-6 text-center">
					<p class="mb-4 text-sm text-red-600">{wrongAnswersError}</p>
					<button
						onclick={loadWrongAnswers}
						class="rounded-lg border border-red-300 bg-white px-4 py-2 text-sm font-medium text-red-700 hover:bg-red-50"
					>
						↺ 다시 시도
					</button>
				</div>
			{:else if wrongAnswers.length === 0}
				<!-- 빈 상태 -->
				<div class="rounded-lg border border-gray-200 bg-white p-12 text-center">
					<p class="text-4xl">📝</p>
					<p class="mt-4 text-base font-medium text-gray-700">아직 틀린 문제가 없어요</p>
					<p class="mt-2 text-sm text-gray-500">퀴즈를 풀고 오답을 아카이빙해보세요.</p>
				</div>
			{:else}
				<!-- 태그 칩 필터 (MVP11 M3) -->
				{#if filterTags.length > 0}
					<div class="mb-4 flex flex-wrap gap-2">
						<button
							onclick={() => (selectedTagId = null)}
							class="rounded-full px-3 py-1 text-sm font-medium transition-colors {selectedTagId === null
								? 'bg-gray-900 text-white'
								: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
						>
							전체
						</button>
						{#each filterTags as tag (tag.id)}
							<button
								onclick={() => (selectedTagId = tag.id)}
								class="rounded-full px-3 py-1 text-sm font-medium transition-colors {selectedTagId === tag.id
									? 'bg-indigo-600 text-white'
									: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
							>
								{tag.name}
							</button>
						{/each}
					</div>
				{/if}
				<div class="space-y-3">
					{#each filteredWrongAnswers as wa (wa.id)}
						<WrongAnswerCard item={wa} onDelete={handleDeleteWrongAnswer} />
					{/each}
				</div>
			{/if}
		{/if}
	</main>
</div>

<!-- MVP9 M2: 기사별 오답 시트 모달 -->
{#if showWrongAnswerSheet}
	<div
		class="fixed inset-0 z-50 flex items-end justify-center bg-black/40 p-4"
		role="dialog"
		aria-modal="true"
		aria-label="오답 보기"
	>
		<div class="w-full max-w-lg rounded-xl bg-white shadow-xl">
			<!-- 헤더 -->
			<div class="flex items-center justify-between border-b border-gray-100 px-5 py-4">
				<h2 class="text-base font-semibold text-gray-900">오답 보기</h2>
				<button
					onclick={closeWrongAnswerSheet}
					aria-label="닫기"
					class="text-gray-400 hover:text-gray-600"
				>
					✕
				</button>
			</div>

			<!-- 본문 -->
			<div class="max-h-[60vh] overflow-y-auto p-5">
				{#if sheetLoading}
					<div class="space-y-3">
						{#each [1, 2] as i (i)}
							<div class="animate-pulse rounded-lg border border-gray-200 bg-gray-50 p-4">
								<div class="mb-2 h-4 w-2/3 rounded bg-gray-200"></div>
								<div class="h-4 w-full rounded bg-gray-200"></div>
							</div>
						{/each}
					</div>
				{:else if sheetWrongAnswers.length === 0}
					<div class="py-8 text-center">
						<p class="text-3xl">🎯</p>
						<p class="mt-3 text-sm font-medium text-gray-700">이 기사의 오답 기록이 없어요</p>
						<p class="mt-1 text-xs text-gray-500">퀴즈를 다시 풀어보세요.</p>
					</div>
				{:else}
					<div class="space-y-3">
						{#each sheetWrongAnswers as wa (wa.id)}
							<!-- 읽기 전용 — 삭제 버튼 없이 WrongAnswerCard 내용 인라인 표시 -->
							<div class="rounded-lg border border-red-100 bg-white p-4 shadow-sm">
								<p class="mb-2 text-xs font-medium text-gray-400 line-clamp-1">{wa.articleTitle}</p>
								<p class="mb-3 text-sm font-semibold text-gray-900 leading-snug">{wa.question}</p>
								<div class="mb-3 space-y-1">
									<div class="flex items-start gap-2 text-xs">
										<span class="flex-shrink-0 rounded bg-red-100 px-1.5 py-0.5 font-medium text-red-700">내 답</span>
										<span class="text-gray-600">{wa.options[wa.userIndex]}</span>
									</div>
									<div class="flex items-start gap-2 text-xs">
										<span class="flex-shrink-0 rounded bg-green-100 px-1.5 py-0.5 font-medium text-green-700">정답</span>
										<span class="text-gray-600">{wa.options[wa.correctIndex]}</span>
									</div>
								</div>
								{#if wa.explanation}
									<p class="text-xs text-gray-500 leading-relaxed">{wa.explanation}</p>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
