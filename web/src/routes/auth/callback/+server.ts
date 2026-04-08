// Apple OAuth 콜백 처리.
// Apple이 code를 쿼리 파라미터로 전달 → Supabase 세션으로 교환 → 홈으로 리다이렉트.

import { redirect } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ url, locals: { supabase } }) => {
	const code = url.searchParams.get('code');

	if (!code) {
		throw redirect(303, '/login');
	}

	const { error } = await supabase.auth.exchangeCodeForSession(code);
	if (error) {
		throw redirect(303, '/login');
	}

	throw redirect(303, '/');
};
