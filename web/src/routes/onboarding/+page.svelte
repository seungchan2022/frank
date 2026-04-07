<script lang="ts">
	import { getAuth } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { apiClient } from '$lib/api';
	import type { Tag } from '$lib/types/tag';

	const auth = getAuth();

	let tags = $state<Tag[]>([]);
	let selectedIds = $state<Set<string>>(new Set());
	let loading = $state(true);
	let saving = $state(false);
	let error = $state('');

	$effect(() => {
		if (!auth.isAuthenticated) {
			goto('/login');
		}
	});

	onMount(async () => {
		try {
			const [allTags, myTagIds] = await Promise.all([
				apiClient.fetchTags(),
				apiClient.fetchMyTagIds()
			]);
			tags = allTags;
			selectedIds = new Set(myTagIds);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load tags';
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

		try {
			await apiClient.saveMyTags([...selectedIds]);
			goto('/feed');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to save tags';
		} finally {
			saving = false;
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

<div class="min-h-screen bg-gray-50 px-4 py-12">
	<div class="mx-auto max-w-2xl space-y-8">
		<div class="text-center">
			<h1 class="text-3xl font-bold text-gray-900">Choose your interests</h1>
			<p class="mt-2 text-gray-600">Select topics you want to follow. We'll curate news for you.</p>
		</div>

		{#if loading}
			<div class="text-center text-gray-500">Loading tags...</div>
		{:else}
			{#if error}
				<div class="rounded-lg bg-red-50 p-3 text-sm text-red-700">{error}</div>
			{/if}

			{#each Object.entries(grouped) as [category, categoryTags]}
				<div>
					<h2 class="mb-3 text-sm font-semibold tracking-wide text-gray-500 uppercase">
						{category}
					</h2>
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

			<div class="pt-4 text-center">
				<button
					onclick={handleSave}
					disabled={saving || selectedIds.size === 0}
					class="rounded-lg bg-blue-600 px-8 py-3 text-white font-medium hover:bg-blue-700 disabled:opacity-50"
				>
					{saving ? 'Saving...' : `Continue (${selectedIds.size} selected)`}
				</button>
			</div>
		{/if}
	</div>
</div>
