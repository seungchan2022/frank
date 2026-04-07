// hooks.server.ts 쿠키 어댑터 + safeGetSession 단위 테스트.
//
// @supabase/ssr는 모킹하지 않는다 — 진짜 createServerClient를 사용해
// 어댑터가 cookies.getAll/setAll을 정확히 호출하는지 검증한다.
// supabase.auth.getSession/getUser만 vi.fn으로 스파이.
//
// SvelteKit `sequence` 헬퍼는 내부 request store를 요구하므로
// supabaseHandle/authHandle을 개별로 호출한다.

import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('$env/static/public', () => ({
	PUBLIC_SUPABASE_URL: 'https://mock.supabase.invalid',
	PUBLIC_SUPABASE_ANON_KEY: 'mock-anon-key'
}));

import { supabaseHandle, authHandle } from './hooks.server';

type CookieEntry = { name: string; value: string; options?: Record<string, unknown> };

function makeEvent(initialCookies: CookieEntry[] = []) {
	const store = new Map<string, CookieEntry>();
	initialCookies.forEach((c) => store.set(c.name, c));

	const cookies = {
		getAll: vi.fn(() =>
			[...store.values()].map(({ name, value }) => ({ name, value }))
		),
		set: vi.fn((name: string, value: string, options?: Record<string, unknown>) => {
			store.set(name, { name, value, options });
		}),
		get: vi.fn((name: string) => store.get(name)?.value),
		delete: vi.fn((name: string) => {
			store.delete(name);
		})
	};

	const event = {
		cookies,
		locals: {} as Record<string, unknown>,
		url: new URL('http://localhost:5173/feed'),
		request: new Request('http://localhost:5173/feed')
	};

	return { event, store };
}

const noopResolve = vi.fn(async () => new Response('ok'));

describe('hooks.server: supabaseHandle', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('locals.supabase가 createServerClient로 초기화된다', async () => {
		const { event } = makeEvent();

		// @ts-expect-error - 테스트에서 최소 event 형태만 제공
		await supabaseHandle({ event, resolve: noopResolve });

		expect(event.locals.supabase).toBeDefined();
		expect((event.locals.supabase as { auth: unknown }).auth).toBeDefined();
		expect(typeof event.locals.safeGetSession).toBe('function');
	});

	it('cookies.getAll 어댑터가 SvelteKit cookies.getAll에 위임한다', async () => {
		const { event } = makeEvent([
			{ name: 'sb-mock-auth-token', value: 'fake.jwt' },
			{ name: 'other', value: 'val' }
		]);

		// @ts-expect-error
		await supabaseHandle({ event, resolve: noopResolve });

		const supabase = event.locals.supabase as {
			auth: { getSession: () => Promise<unknown> };
		};
		await supabase.auth.getSession().catch(() => undefined);

		// @supabase/ssr가 쿠키를 읽으려고 어댑터를 호출 → SvelteKit cookies.getAll로 위임됨
		expect(event.cookies.getAll).toHaveBeenCalled();
	});

	it('safeGetSession은 session 없을 때 { null, null } 반환', async () => {
		const { event } = makeEvent();

		// @ts-expect-error
		await supabaseHandle({ event, resolve: noopResolve });

		const supabase = event.locals.supabase as {
			auth: {
				getSession: () => Promise<{ data: { session: null } }>;
				getUser: () => Promise<unknown>;
			};
		};
		supabase.auth.getSession = vi.fn().mockResolvedValue({ data: { session: null } });
		supabase.auth.getUser = vi.fn();

		const result = await (
			event.locals.safeGetSession as () => Promise<{ session: null; user: null }>
		)();

		expect(result).toEqual({ session: null, user: null });
		// session이 없으면 getUser는 호출되지 않아야 함 (불필요한 네트워크 절약)
		expect(supabase.auth.getUser).not.toHaveBeenCalled();
	});

	it('safeGetSession은 getSession 성공 + getUser 실패 시 세션 무효 처리', async () => {
		const { event } = makeEvent();

		// @ts-expect-error
		await supabaseHandle({ event, resolve: noopResolve });

		const supabase = event.locals.supabase as {
			auth: {
				getSession: () => Promise<{ data: { session: unknown } }>;
				getUser: () => Promise<{
					data: { user: unknown };
					error: { message: string } | null;
				}>;
			};
		};
		supabase.auth.getSession = vi.fn().mockResolvedValue({
			data: { session: { access_token: 'fake' } }
		});
		supabase.auth.getUser = vi.fn().mockResolvedValue({
			data: { user: null },
			error: { message: 'JWT verification failed' }
		});

		const result = await (
			event.locals.safeGetSession as () => Promise<{
				session: unknown;
				user: unknown;
			}>
		)();

		expect(result).toEqual({ session: null, user: null });
		expect(supabase.auth.getUser).toHaveBeenCalled();
	});

	it('safeGetSession은 getSession + getUser 모두 성공 시 { session, user } 반환', async () => {
		const { event } = makeEvent();

		// @ts-expect-error
		await supabaseHandle({ event, resolve: noopResolve });

		const fakeSession = { access_token: 'fake.jwt', user: { id: 'u1' } };
		const fakeUser = { id: 'u1', email: 'test@example.invalid' };
		const supabase = event.locals.supabase as {
			auth: {
				getSession: () => Promise<{ data: { session: typeof fakeSession } }>;
				getUser: () => Promise<{
					data: { user: typeof fakeUser };
					error: null;
				}>;
			};
		};
		supabase.auth.getSession = vi
			.fn()
			.mockResolvedValue({ data: { session: fakeSession } });
		supabase.auth.getUser = vi
			.fn()
			.mockResolvedValue({ data: { user: fakeUser }, error: null });

		const result = await (
			event.locals.safeGetSession as () => Promise<{
				session: typeof fakeSession;
				user: typeof fakeUser;
			}>
		)();

		expect(result).toEqual({ session: fakeSession, user: fakeUser });
	});
});

describe('hooks.server: authHandle', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('safeGetSession 결과가 locals.session/user에 채워진다', async () => {
		const { event } = makeEvent();
		// supabaseHandle을 먼저 통과시켜 locals.safeGetSession 주입
		// @ts-expect-error
		await supabaseHandle({ event, resolve: noopResolve });

		// safeGetSession을 결정적 mock으로 대체
		const fakeSession = { access_token: 'token' };
		const fakeUser = { id: 'u1' };
		event.locals.safeGetSession = vi.fn().mockResolvedValue({
			session: fakeSession,
			user: fakeUser
		});

		// @ts-expect-error
		await authHandle({ event, resolve: noopResolve });

		expect(event.locals.session).toBe(fakeSession);
		expect(event.locals.user).toBe(fakeUser);
	});

	it('safeGetSession이 null 반환 시 locals.session/user도 null', async () => {
		const { event } = makeEvent();
		// @ts-expect-error
		await supabaseHandle({ event, resolve: noopResolve });

		event.locals.safeGetSession = vi.fn().mockResolvedValue({
			session: null,
			user: null
		});

		// @ts-expect-error
		await authHandle({ event, resolve: noopResolve });

		expect(event.locals.session).toBeNull();
		expect(event.locals.user).toBeNull();
	});
});
