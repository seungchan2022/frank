// Server-driven auth store 단위 테스트.
//
// M2 ST-4 (옵션 B): client-side supabase 호출 제거.
// store는 +layout.svelte의 setAuth(...) 호출로만 hydration된다.

import { describe, it, expect, beforeEach, vi } from 'vitest';

let getAuth: typeof import('./auth.svelte').getAuth;
let setAuth: typeof import('./auth.svelte').setAuth;

beforeEach(async () => {
	vi.resetModules();
	const mod = await import('./auth.svelte');
	getAuth = mod.getAuth;
	setAuth = mod.setAuth;
});

describe('getAuth (initial)', () => {
	it('returns null session/user, isAuthenticated false', () => {
		const auth = getAuth();
		expect(auth.session).toBeNull();
		expect(auth.user).toBeNull();
		expect(auth.isAuthenticated).toBe(false);
		expect(auth.loading).toBe(false);
	});
});

describe('setAuth', () => {
	it('hydrates session + user → isAuthenticated true', () => {
		const session = { access_token: 'tok', user: { id: 'u1', email: 'test@test.com' } };
		const user = { id: 'u1', email: 'test@test.com' };

		// @ts-expect-error - 테스트에서 최소 Session/User 형태만 제공
		setAuth({ session, user });

		const auth = getAuth();
		expect(auth.session).toEqual(session);
		expect(auth.user).toEqual(user);
		expect(auth.isAuthenticated).toBe(true);
		expect(auth.loading).toBe(false);
	});

	it('null session 전달 시 isAuthenticated false로 복귀', () => {
		// 먼저 인증 상태로
		// @ts-expect-error
		setAuth({ session: { access_token: 't' }, user: { id: 'u' } });
		expect(getAuth().isAuthenticated).toBe(true);

		// null로 reset
		setAuth({ session: null, user: null });
		const auth = getAuth();
		expect(auth.session).toBeNull();
		expect(auth.user).toBeNull();
		expect(auth.isAuthenticated).toBe(false);
	});

	it('isAuthenticated는 session 존재 여부로만 판정', () => {
		// session 있고 user null인 경우 (이상하지만 가능)
		// @ts-expect-error
		setAuth({ session: { access_token: 't' }, user: null });
		expect(getAuth().isAuthenticated).toBe(true);

		// session null이면 user 있어도 false
		// @ts-expect-error
		setAuth({ session: null, user: { id: 'u' } });
		expect(getAuth().isAuthenticated).toBe(false);
	});
});
