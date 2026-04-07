<script lang="ts">
	import { enhance } from '$app/forms';
	import type { ActionData } from './$types';

	let { form }: { form: ActionData } = $props();

	let isSignUp = $state(false);
	let loading = $state(false);

	const signUpSuccess = $derived(form?.signUpSuccess === true);
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
			<form
				method="POST"
				action={isSignUp ? '?/signup' : '?/signin'}
				class="space-y-4"
				use:enhance={() => {
					loading = true;
					return async ({ update }) => {
						await update();
						loading = false;
					};
				}}
			>
				{#if form?.error}
					<div class="rounded-lg bg-red-50 p-3 text-sm text-red-700">{form.error}</div>
				{/if}

				<div>
					<label for="email" class="block text-sm font-medium text-gray-700">Email</label>
					<input
						id="email"
						name="email"
						type="email"
						value={form?.email ?? ''}
						required
						class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500 focus:outline-none"
						placeholder="you@example.com"
					/>
				</div>

				<div>
					<label for="password" class="block text-sm font-medium text-gray-700">Password</label>
					<input
						id="password"
						name="password"
						type="password"
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
					type="button"
					class="ml-1 font-medium text-blue-600 hover:text-blue-500"
					onclick={() => {
						isSignUp = !isSignUp;
					}}
				>
					{isSignUp ? 'Sign In' : 'Sign Up'}
				</button>
			</p>
		{/if}
	</div>
</div>
