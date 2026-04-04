<script lang="ts">
	import { getAuth } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { fetchTags, fetchMyTagIds, updateMyTags } from '$lib/utils/api';
	import type { Tag } from '$lib/types/tag';
	import Header from '$lib/components/Header.svelte';

	const auth = getAuth();

	let tags = $state<Tag[]>([]);
	let selectedIds = $state<Set<string>>(new Set());
	let savedIds = $state<Set<string>>(new Set());
	let loading = $state(true);
	let saving = $state(false);
	let error = $state('');
	let success = $state('');

	const hasChanges = $derived(
		selectedIds.size !== savedIds.size ||
			[...selectedIds].some((id) => !savedIds.has(id))
	);

	$effect(() => {
		if (!auth.isAuthenticated) {
			goto('/login');
		}
	});

	onMount(async () => {
		try {
			const [allTags, myTagIds] = await Promise.all([fetchTags(), fetchMyTagIds()]);
			tags = allTags;
			selectedIds = new Set(myTagIds);
			savedIds = new Set(myTagIds);
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
		success = '';

		try {
			await updateMyTags([...selectedIds]);
			savedIds = new Set(selectedIds);
			success = 'Tags saved successfully.';
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

<div class="min-h-screen bg-gray-50">
	<Header />

	<main class="mx-auto max-w-2xl px-6 py-8">
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
