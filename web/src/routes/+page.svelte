<script lang="ts">
	import { getAuth } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { fetchProfile } from '$lib/utils/api';

	const auth = getAuth();

	onMount(async () => {
		if (!auth.isAuthenticated) {
			goto('/login');
			return;
		}

		try {
			const profile = await fetchProfile();
			if (!profile.onboarding_completed) {
				goto('/onboarding');
			} else {
				goto('/feed');
			}
		} catch {
			goto('/onboarding');
		}
	});
</script>

<div class="flex h-screen items-center justify-center">
	<p class="text-gray-500">Redirecting...</p>
</div>
