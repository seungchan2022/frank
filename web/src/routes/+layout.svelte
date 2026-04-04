<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { initAuth, getAuth } from '$lib/stores/auth.svelte';
	import { onMount } from 'svelte';

	let { children } = $props();
	const auth = getAuth();

	onMount(() => {
		initAuth();
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{#if auth.loading}
	<div class="flex h-screen items-center justify-center">
		<p class="text-gray-500">Loading...</p>
	</div>
{:else}
	{@render children()}
{/if}
