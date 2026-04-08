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

			<div class="relative flex items-center py-2">
				<div class="flex-grow border-t border-gray-200"></div>
				<span class="mx-3 flex-shrink text-xs text-gray-400">또는</span>
				<div class="flex-grow border-t border-gray-200"></div>
			</div>

			<form method="POST" action="?/appleOAuth">
				<button
					type="submit"
					class="flex w-full items-center justify-center gap-2 rounded-lg border border-gray-300 bg-white py-2.5 font-medium text-gray-800 hover:bg-gray-50"
				>
					<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
						<path
							d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.8-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"
						/>
					</svg>
					Apple로 계속하기
				</button>
			</form>
		{/if}
	</div>
</div>
