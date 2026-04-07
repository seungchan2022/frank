// Root /의 server-side redirect.
// session 유무 + onboarding_completed에 따라 /login, /onboarding, /feed로 분기.

import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

const RUST_API_BASE = (import.meta.env.VITE_RUST_API_URL ?? 'http://localhost:8080').replace(
	/\/$/,
	''
);

export const load: PageServerLoad = async ({ locals: { session }, fetch }) => {
	if (!session) {
		throw redirect(303, '/login');
	}

	// Rust API로 프로필 조회 → onboarding 분기.
	// 404 또는 onboarding_completed=false → /onboarding
	// 그 외 정상 → /feed
	let onboardingCompleted = false;
	try {
		const res = await fetch(`${RUST_API_BASE}/api/me/profile`, {
			headers: {
				Authorization: `Bearer ${session.access_token}`,
				'Content-Type': 'application/json'
			}
		});
		if (res.ok) {
			const profile = (await res.json()) as { onboarding_completed?: boolean };
			onboardingCompleted = profile.onboarding_completed === true;
		}
		// 404 또는 5xx → onboardingCompleted=false (안전한 기본값)
	} catch {
		// 네트워크 에러 → 온보딩으로 보내서 사용자가 다시 시도
	}

	if (!onboardingCompleted) {
		throw redirect(303, '/onboarding');
	}
	throw redirect(303, '/feed');
};
