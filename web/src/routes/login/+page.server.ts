// Server-side login/signup form actions.
// 클라이언트는 <form method="POST" use:enhance>로 호출.
// 모든 인증은 server에서 supabase로 처리 → cookie httpOnly: true 강제 (hooks.server.ts).

import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals: { session } }) => {
	if (session) {
		// 이미 로그인됨 — /로 리다이렉트 (root에서 onboarding/feed 분기)
		throw redirect(303, '/');
	}
	return {};
};

export const actions: Actions = {
	signin: async ({ request, locals: { supabase } }) => {
		const formData = await request.formData();
		const email = String(formData.get('email') ?? '').trim();
		const password = String(formData.get('password') ?? '');

		if (!email || !password) {
			return fail(400, { email, error: 'Email and password are required.' });
		}

		const { error } = await supabase.auth.signInWithPassword({ email, password });
		if (error) {
			return fail(400, { email, error: error.message });
		}

		throw redirect(303, '/');
	},

	signup: async ({ request, locals: { supabase } }) => {
		const formData = await request.formData();
		const email = String(formData.get('email') ?? '').trim();
		const password = String(formData.get('password') ?? '');

		if (!email || !password) {
			return fail(400, { email, error: 'Email and password are required.' });
		}

		const { error } = await supabase.auth.signUp({ email, password });
		if (error) {
			return fail(400, { email, signUp: true, error: error.message });
		}

		return { signUpSuccess: true, email };
	}
};
