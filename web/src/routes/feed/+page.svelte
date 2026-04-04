<script lang="ts">
	import { getAuth, signOut } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';

	const auth = getAuth();

	$effect(() => {
		if (!auth.isAuthenticated) {
			goto('/login');
		}
	});

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

	<main class="mx-auto max-w-4xl px-6 py-12">
		<div class="text-center text-gray-500">
			<p class="text-lg">Your feed will appear here.</p>
			<p class="mt-2 text-sm">Coming in M2: Web search + collection pipeline</p>
		</div>
	</main>
</div>
