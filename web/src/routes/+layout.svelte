<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { setAuth, getAuth } from '$lib/stores/auth.svelte';
	import { setRealClientToken } from '$lib/api/realClient';

	let { children, data } = $props();
	const auth = getAuth();

	// $effect.pre는 DOM update + 자식 onMount보다 먼저 실행된다.
	// page.data 변경 시마다 auth store + realClient token 동기화.
	$effect.pre(() => {
		setAuth({ session: data.session, user: data.user });
		setRealClientToken(data.session?.access_token ?? null);
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
