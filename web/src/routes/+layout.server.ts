// SSR 시점에 세션을 페이지 데이터로 hydration.
// 진실의 원천: hooks.server.ts의 event.locals.safeGetSession()
//
// 클라이언트 +layout.ts가 이 데이터를 받아 createBrowserClient를 초기화한다.

import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

const PUBLIC_PATHS = new Set(['/login']);

export const load: LayoutServerLoad = async ({ locals: { safeGetSession }, cookies, url }) => {
	const { session, user } = await safeGetSession();

	// 전역 인증 가드: 미인증 + 비공개 경로 → /login으로 redirect.
	// /login은 공개 경로이지만 이미 인증된 상태에서 진입 시 /로 보낸다 (login/+page.server.ts에서 처리).
	if (!session && !PUBLIC_PATHS.has(url.pathname)) {
		throw redirect(303, '/login');
	}

	// session 전체를 page.data로 전달 — Rust API 직통 호출 시 Bearer 토큰으로 사용.
	// httpOnly cookie와 trade-off: client JS가 access_token을 봐야 fetch에 첨부 가능.
	// SvelteKit endpoint proxy 패턴은 ST-5에서 제거됐으므로 직통 호출을 유지한다.
	return {
		session,
		user,
		cookies: cookies.getAll()
	};
};
