<script lang="ts">
	import { getAuth, signInWithEmail, signUpWithEmail } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';

	const auth = getAuth();

	let email = $state('');
	let password = $state('');
	let isSignUp = $state(false);
	let error = $state('');
	let loading = $state(false);
	let signUpSuccess = $state(false);

	$effect(() => {
		if (auth.isAuthenticated) {
			goto('/');
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		loading = true;

		try {
			if (isSignUp) {
				await signUpWithEmail(email, password);
				signUpSuccess = true;
			} else {
				await signInWithEmail(email, password);
				goto('/');
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An error occurred';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-gray-50 px-4">
	<div class="w-full max-w-md space-y-8">
		<div class="text-center">
			<h1 class="text-3xl font-bold text-gray-900">Frank</h1>
			<p class="mt-2 text-gray-600">AI News Scrap Study App</p>
		</div>

		{#if signUpSuccess}
			<div class="rounded-lg bg-green-50 p-4 text-center text-green-800">
				<p class="font-medium">Check your email to confirm your account.</p>
			</div>
		{:else}
			<form class="space-y-4" onsubmit={handleSubmit}>
				{#if error}
					<div class="rounded-lg bg-red-50 p-3 text-sm text-red-700">{error}</div>
				{/if}

				<div>
					<label for="email" class="block text-sm font-medium text-gray-700">Email</label>
					<input
						id="email"
						type="email"
						bind:value={email}
						required
						class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500 focus:outline-none"
						placeholder="you@example.com"
					/>
				</div>

				<div>
					<label for="password" class="block text-sm font-medium text-gray-700">Password</label>
					<input
						id="password"
						type="password"
						bind:value={password}
						required
						minlength="6"
						class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500 focus:outline-none"
						placeholder="6+ characters"
					/>
				</div>

				<button
					type="submit"
					disabled={loading}
					class="w-full rounded-lg bg-blue-600 py-2.5 text-white font-medium hover:bg-blue-700 disabled:opacity-50"
				>
					{loading ? 'Loading...' : isSignUp ? 'Sign Up' : 'Sign In'}
				</button>
			</form>

			<p class="text-center text-sm text-gray-600">
				{isSignUp ? 'Already have an account?' : "Don't have an account?"}
				<button
					class="ml-1 font-medium text-blue-600 hover:text-blue-500"
					onclick={() => {
						isSignUp = !isSignUp;
						error = '';
					}}
				>
					{isSignUp ? 'Sign In' : 'Sign Up'}
				</button>
			</p>
		{/if}
	</div>
</div>
