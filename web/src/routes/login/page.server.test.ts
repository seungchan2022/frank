// routes/login/+page.server.ts form actions 단위 테스트.
// supabase는 vi.fn으로 mock. SvelteKit redirect/fail 헬퍼는 진짜 사용 (vitest 환경에서 동작).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { actions, load } from './+page.server';

const mockSignInWithPassword = vi.fn();
const mockSignUp = vi.fn();

function makeLocals(session: unknown = null) {
	return {
		session,
		user: null,
		safeGetSession: vi.fn(),
		supabase: {
			auth: {
				signInWithPassword: mockSignInWithPassword,
				signUp: mockSignUp
			}
		}
	};
}

function makeRequest(form: Record<string, string>): Request {
	const fd = new FormData();
	for (const [k, v] of Object.entries(form)) fd.set(k, v);
	return new Request('http://localhost/login', { method: 'POST', body: fd });
}

beforeEach(() => {
	vi.clearAllMocks();
});

describe('load', () => {
	it('미인증 상태면 빈 객체 반환', async () => {
		// @ts-expect-error - 최소 형태
		const result = await load({ locals: makeLocals(null) });
		expect(result).toEqual({});
	});

	it('이미 인증된 상태면 / 로 redirect', async () => {
		await expect(
			// @ts-expect-error
			load({ locals: makeLocals({ access_token: 't' }) })
		).rejects.toMatchObject({ status: 303, location: '/' });
	});
});

describe('actions.signin', () => {
	it('email/password 누락 시 fail 400', async () => {
		const result = await actions.signin({
			// @ts-expect-error
			request: makeRequest({ email: '', password: '' }),
			locals: makeLocals()
		});
		expect(result).toMatchObject({ status: 400, data: { error: expect.any(String) } });
	});

	it('signInWithPassword 성공 → / 로 redirect', async () => {
		mockSignInWithPassword.mockResolvedValue({ error: null });

		await expect(
			actions.signin({
				// @ts-expect-error
				request: makeRequest({ email: 'a@b.com', password: 'pw123456' }),
				locals: makeLocals()
			})
		).rejects.toMatchObject({ status: 303, location: '/' });

		expect(mockSignInWithPassword).toHaveBeenCalledWith({
			email: 'a@b.com',
			password: 'pw123456'
		});
	});

	it('signInWithPassword 실패 → fail 400 + error 메시지', async () => {
		mockSignInWithPassword.mockResolvedValue({
			error: { message: 'Invalid login credentials' }
		});

		const result = await actions.signin({
			// @ts-expect-error
			request: makeRequest({ email: 'a@b.com', password: 'wrong' }),
			locals: makeLocals()
		});

		expect(result).toMatchObject({
			status: 400,
			data: { error: 'Invalid login credentials', email: 'a@b.com' }
		});
	});

	it('email은 trim 후 supabase에 전달', async () => {
		mockSignInWithPassword.mockResolvedValue({ error: null });

		await expect(
			actions.signin({
				// @ts-expect-error
				request: makeRequest({ email: '  a@b.com  ', password: 'pw' }),
				locals: makeLocals()
			})
		).rejects.toMatchObject({ status: 303 });

		expect(mockSignInWithPassword).toHaveBeenCalledWith({
			email: 'a@b.com',
			password: 'pw'
		});
	});
});

describe('actions.signup', () => {
	it('성공 시 signUpSuccess 반환 (redirect 안 함)', async () => {
		mockSignUp.mockResolvedValue({ error: null });

		const result = await actions.signup({
			// @ts-expect-error
			request: makeRequest({ email: 'new@test.com', password: 'pw123456' }),
			locals: makeLocals()
		});

		expect(result).toEqual({ signUpSuccess: true, email: 'new@test.com' });
	});

	it('실패 시 fail 400', async () => {
		mockSignUp.mockResolvedValue({ error: { message: 'Already exists' } });

		const result = await actions.signup({
			// @ts-expect-error
			request: makeRequest({ email: 'dup@test.com', password: 'pw123456' }),
			locals: makeLocals()
		});

		expect(result).toMatchObject({
			status: 400,
			data: { error: 'Already exists', signUp: true }
		});
	});

	it('email/password 누락 시 fail 400', async () => {
		const result = await actions.signup({
			// @ts-expect-error
			request: makeRequest({ email: '', password: '' }),
			locals: makeLocals()
		});
		expect(result).toMatchObject({ status: 400 });
	});
});
