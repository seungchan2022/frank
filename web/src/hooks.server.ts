// SvelteKit server hook — Supabase SSR + httpOnly 쿠키 세션.
// 진실의 원천 패턴: https://supabase.com/docs/guides/auth/server-side/sveltekit
//
// M2 ST-4: localStorage 세션을 httpOnly 쿠키로 전환.
// safeGetSession()은 getSession() + getUser() 이중 검증으로 토큰 변조를 방지한다.

import { createServerClient } from '@supabase/ssr';
import type { Handle } from '@sveltejs/kit';
import { sequence } from '@sveltejs/kit/hooks';
import { PUBLIC_SUPABASE_URL, PUBLIC_SUPABASE_ANON_KEY } from '$env/static/public';

export const supabaseHandle: Handle = async ({ event, resolve }) => {
	event.locals.supabase = createServerClient(PUBLIC_SUPABASE_URL, PUBLIC_SUPABASE_ANON_KEY, {
		cookies: {
			getAll: () => event.cookies.getAll(),
			setAll: (cookiesToSet) => {
				cookiesToSet.forEach(({ name, value, options }) => {
					// httpOnly: true 강제 — client JS의 document.cookie 접근 차단.
					// access_token은 +layout.server.ts에서 page.data로 별도 전달.
					event.cookies.set(name, value, {
						...options,
						path: '/',
						httpOnly: true,
						sameSite: 'lax'
					});
				});
			}
		}
	});

	/**
	 * safeGetSession: getSession() (쿠키 디코드, 변조 검증 X) +
	 * getUser() (Supabase Auth 서버에 검증 요청)을 모두 호출하여
	 * 안전한 session을 반환한다.
	 */
	event.locals.safeGetSession = async () => {
		const {
			data: { session }
		} = await event.locals.supabase.auth.getSession();
		if (!session) {
			return { session: null, user: null };
		}

		const {
			data: { user },
			error
		} = await event.locals.supabase.auth.getUser();
		if (error) {
			// JWT 검증 실패 → 세션 무효
			return { session: null, user: null };
		}

		return { session, user };
	};

	return resolve(event, {
		filterSerializedResponseHeaders: (name) => {
			return name === 'content-range' || name === 'x-supabase-api-version';
		}
	});
};

export const authHandle: Handle = async ({ event, resolve }) => {
	const { session, user } = await event.locals.safeGetSession();
	event.locals.session = session;
	event.locals.user = user;
	return resolve(event);
};

export const handle: Handle = sequence(supabaseHandle, authHandle);
