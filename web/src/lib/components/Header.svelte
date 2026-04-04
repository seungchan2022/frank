<script lang="ts">
	import { getAuth, signOut } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	const auth = getAuth();

	const currentPath = $derived(page.url.pathname);

	async function handleSignOut() {
		await signOut();
		goto('/login');
	}

	function isActive(path: string): boolean {
		if (path === '/feed') {
			return currentPath === '/feed' || currentPath.startsWith('/feed/');
		}
		return currentPath === path;
	}
</script>

<header class="border-b bg-white px-6 py-4">
	<div class="mx-auto flex max-w-4xl items-center justify-between">
		<div class="flex items-center gap-6">
			<a href="/feed" class="text-xl font-bold text-gray-900">Frank</a>
			<nav class="flex items-center gap-4">
				<a
					href="/feed"
					class="text-sm font-medium {isActive('/feed')
						? 'text-blue-600'
						: 'text-gray-600 hover:text-gray-900'}"
				>
					Feed
				</a>
				<a
					href="/settings"
					class="text-sm font-medium {isActive('/settings')
						? 'text-blue-600'
						: 'text-gray-600 hover:text-gray-900'}"
				>
					Settings
				</a>
			</nav>
		</div>
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
