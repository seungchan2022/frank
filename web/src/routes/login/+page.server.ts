// Server-side login/signup form actions.
// 클라이언트는 <form method="POST" use:enhance>로 호출.
// 모든 인증은 server에서 supabase로 처리 → cookie httpOnly: true 강제 (hooks.server.ts).

import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad, RequestEvent } from './$types';

/** Supabase 인증 에러 메시지를 사용자 친화적인 한국어로 변환 */
function toUserMessage(message: string): string {
	const lower = message.toLowerCase();
	if (lower.includes('invalid login credentials') || lower.includes('invalid credentials')) {
		return '이메일 또는 비밀번호가 틀렸어요.';
	}
	if (lower.includes('email') && lower.includes('invalid')) {
		return '사용할 수 없는 이메일 주소예요. Gmail, Naver 등 실제 이메일을 사용해주세요.';
	}
	if (lower.includes('rate limit') || lower.includes('too many')) {
		return '잠시 후 다시 시도해주세요. (이메일 발송 횟수 초과)';
	}
	if (lower.includes('user already registered') || lower.includes('already been registered')) {
		return '이미 가입된 이메일이에요. 로그인을 시도해보세요.';
	}
	if (lower.includes('email not confirmed')) {
		return '이메일 인증이 필요해요. 가입 시 받은 인증 메일을 확인해주세요.';
	}
	if (lower.includes('password') && lower.includes('least')) {
		return '비밀번호는 6자 이상이어야 해요.';
	}
	if (lower.includes('signup') && lower.includes('disabled')) {
		return '현재 회원가입이 비활성화되어 있어요.';
	}
	return '오류가 발생했어요. 잠시 후 다시 시도해주세요.';
}

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
			return fail(400, { email, error: toUserMessage(error.message) });
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
			return fail(400, { email, signUp: true, error: toUserMessage(error.message) });
		}

		return { signUpSuccess: true, email };
	},

	appleOAuth: async ({ url, locals: { supabase } }: RequestEvent) => {
		const origin = url.origin;

		const { data, error } = await supabase.auth.signInWithOAuth({
			provider: 'apple',
			options: {
				redirectTo: `${origin}/auth/callback`,
				scopes: 'email name'
			}
		});

		if (error || !data.url) {
			throw redirect(302, '/login');
		}

		throw redirect(302, data.url);
	}
};
