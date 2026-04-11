// GET /api/auth/token — 현재 세션의 access_token 반환.
// httpOnly 쿠키는 client JS에서 접근 불가이므로, realClient.ts가
// 401(토큰 만료) 수신 시 이 엔드포인트를 경유해 토큰을 갱신한다.
// safeGetSession()이 내부적으로 Supabase SDK를 통해 refresh를 처리한다.

import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals: { safeGetSession } }) => {
	const { session } = await safeGetSession();
	if (!session) {
		return json({ token: null }, { status: 401 });
	}
	return json({ token: session.access_token });
};
