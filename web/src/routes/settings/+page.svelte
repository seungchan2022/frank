<script lang="ts">
	import { getAuth } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { apiClient } from '$lib/api';
	import type { Tag } from '$lib/types/tag';
	import Header from '$lib/components/Header.svelte';
	import { feedStore } from '$lib/stores/feedStore.svelte';

	const auth = getAuth();

	let tags = $state<Tag[]>([]);
	let selectedIds = $state<Set<string>>(new Set());
	let savedIds = $state<Set<string>>(new Set());
	let loading = $state(true);
	let saving = $state(false);
	let error = $state('');
	let success = $state('');

	// MVP15 M3: occupation 상태
	let occupation = $state<string>('');
	let savedOccupation = $state<string>('');
	let savingOccupation = $state(false);
	let occupationError = $state('');
	let occupationSuccess = $state('');

	const OCCUPATION_MAX = 50;

	const hasChanges = $derived(
		selectedIds.size !== savedIds.size ||
			[...selectedIds].some((id) => !savedIds.has(id))
	);

	const hasOccupationChanges = $derived(occupation !== savedOccupation);

	$effect(() => {
		if (!auth.isAuthenticated) {
			goto('/login');
		}
	});

	onMount(async () => {
		try {
			const [allTags, myTagIds, profile] = await Promise.all([
				apiClient.fetchTags(),
				apiClient.fetchMyTagIds(),
				apiClient.fetchProfile()
			]);
			tags = allTags;
			selectedIds = new Set(myTagIds);
			savedIds = new Set(myTagIds);
			const occ = profile.occupation ?? '';
			occupation = occ;
			savedOccupation = occ;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load settings';
		} finally {
			loading = false;
		}
	});

	function toggleTag(tagId: string) {
		const next = new Set(selectedIds);
		if (next.has(tagId)) {
			next.delete(tagId);
		} else {
			next.add(tagId);
		}
		selectedIds = next;
	}

	async function handleSave() {
		if (selectedIds.size === 0) {
			error = 'Please select at least one tag.';
			return;
		}

		saving = true;
		error = '';
		success = '';

		try {
			await apiClient.updateMyTags([...selectedIds]);
			savedIds = new Set(selectedIds);
			success = 'Tags saved successfully.';
			// 태그 변경 → 피드 캐시 무효화 (다음 피드 방문 시 재로드)
			feedStore.reset();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to save tags';
		} finally {
			saving = false;
		}
	}

	async function handleSaveOccupation() {
		const trimmed = occupation.trim();
		if (trimmed.length > OCCUPATION_MAX) {
			occupationError = `직업은 최대 ${OCCUPATION_MAX}자까지 입력할 수 있습니다.`;
			return;
		}

		savingOccupation = true;
		occupationError = '';
		occupationSuccess = '';

		try {
			await apiClient.updateProfile({ occupation: trimmed.length > 0 ? trimmed : null });
			savedOccupation = trimmed;
			occupation = trimmed;
			occupationSuccess = '직업이 저장되었습니다.';
		} catch (err) {
			occupationError = err instanceof Error ? err.message : '저장에 실패했습니다.';
		} finally {
			savingOccupation = false;
		}
	}

	const grouped = $derived(
		tags.reduce<Record<string, Tag[]>>((acc, tag) => {
			const cat = tag.category ?? 'Other';
			if (!acc[cat]) acc[cat] = [];
			acc[cat].push(tag);
			return acc;
		}, {})
	);
</script>

<div class="min-h-screen bg-gray-50">
	<Header />

	<main class="mx-auto max-w-2xl px-6 py-8">
		<!-- MVP15 M3: 직업 입력 섹션 -->
		<section class="mb-10">
			<h2 class="mb-1 text-lg font-semibold text-gray-900">내 직업</h2>
			<p class="mb-4 text-sm text-gray-500">
				직업을 입력하면 기사 요약 시 직업 시각 인사이트를 받고, 재작성 기능을 이용할 수 있습니다.
			</p>

			{#if occupationError}
				<div class="mb-3 rounded-lg bg-red-50 p-3 text-sm text-red-700">{occupationError}</div>
			{/if}
			{#if occupationSuccess}
				<div class="mb-3 rounded-lg bg-green-50 p-3 text-sm text-green-700">{occupationSuccess}</div>
			{/if}

			<div class="flex items-center gap-3">
				<div class="relative flex-1">
					<input
						type="text"
						bind:value={occupation}
						maxlength={OCCUPATION_MAX}
						placeholder="예: iOS 개발자, 백엔드 엔지니어, 데이터 분석가"
						class="w-full rounded-lg border border-gray-300 px-4 py-2.5 text-sm text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 focus:outline-none"
						disabled={loading || savingOccupation}
					/>
					<span
						class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-gray-400"
					>{occupation.length}/{OCCUPATION_MAX}</span>
				</div>
				<button
					onclick={handleSaveOccupation}
					disabled={savingOccupation || !hasOccupationChanges || loading}
					class="rounded-lg bg-blue-600 px-5 py-2.5 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
				>
					{savingOccupation ? '저장 중...' : '저장'}
				</button>
			</div>
		</section>

		<h2 class="mb-2 text-lg font-semibold text-gray-900">Manage Keywords</h2>
		<p class="mb-6 text-sm text-gray-500">
			Select topics you want to follow. Changes will take effect on the next collection.
		</p>

		{#if loading}
			<div class="text-center text-gray-500">Loading tags...</div>
		{:else}
			{#if error}
				<div class="mb-4 rounded-lg bg-red-50 p-3 text-sm text-red-700">{error}</div>
			{/if}
			{#if success}
				<div class="mb-4 rounded-lg bg-green-50 p-3 text-sm text-green-700">{success}</div>
			{/if}

			<div class="space-y-6">
				{#each Object.entries(grouped) as [category, categoryTags]}
					<div>
						<h3 class="mb-3 text-sm font-semibold tracking-wide text-gray-500 uppercase">
							{category}
						</h3>
						<div class="flex flex-wrap gap-2">
							{#each categoryTags as tag}
								<button
									onclick={() => toggleTag(tag.id)}
									class="rounded-full border px-4 py-2 text-sm font-medium transition-colors {selectedIds.has(
										tag.id
									)
										? 'border-blue-600 bg-blue-600 text-white'
										: 'border-gray-300 bg-white text-gray-700 hover:border-blue-400'}"
								>
									{tag.name}
								</button>
							{/each}
						</div>
					</div>
				{/each}
			</div>

			<div class="mt-8 flex items-center gap-4">
				<button
					onclick={handleSave}
					disabled={saving || selectedIds.size === 0 || !hasChanges}
					class="rounded-lg bg-blue-600 px-6 py-2.5 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
				>
					{saving ? 'Saving...' : `Save (${selectedIds.size} selected)`}
				</button>
				{#if !hasChanges && !saving}
					<span class="text-xs text-gray-400">No changes</span>
				{/if}
			</div>
		{/if}
	</main>
</div>
