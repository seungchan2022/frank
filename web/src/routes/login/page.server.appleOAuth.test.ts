// routes/login/+page.server.ts appleOAuth 액션 단위 테스트.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { actions } from './+page.server';

const mockSignInWithOAuth = vi.fn();

function makeLocals() {
	return {
		session: null,
		user: null,
		safeGetSession: vi.fn(),
		supabase: {
			auth: {
				signInWithOAuth: mockSignInWithOAuth
			}
		}
	};
}

function makeRequest(origin = 'https://example.com'): Request {
	return new Request(`${origin}/login`, { method: 'POST' });
}

beforeEach(() => {
	vi.clearAllMocks();
});

describe('actions.appleOAuth', () => {
	it('signInWithOAuth 성공 → url로 302 redirect', async () => {
		const redirectUrl = 'https://accounts.apple.com/authorize?...';
		mockSignInWithOAuth.mockResolvedValue({
			data: { url: redirectUrl },
			error: null
		});

		await expect(
			actions.appleOAuth({
				// @ts-expect-error - 최소 형태
				request: makeRequest('https://example.com'),
				url: new URL('https://example.com/login'),
				locals: makeLocals()
			})
		).rejects.toMatchObject({ status: 302, location: redirectUrl });

		expect(mockSignInWithOAuth).toHaveBeenCalledWith({
			provider: 'apple',
			options: {
				redirectTo: 'https://example.com/auth/callback',
				scopes: 'email name'
			}
		});
	});

	it('signInWithOAuth url이 null이면 /login으로 302 redirect', async () => {
		mockSignInWithOAuth.mockResolvedValue({
			data: { url: null },
			error: null
		});

		await expect(
			actions.appleOAuth({
				// @ts-expect-error
				request: makeRequest(),
				url: new URL('https://example.com/login'),
				locals: makeLocals()
			})
		).rejects.toMatchObject({ status: 302, location: '/login' });
	});

	it('signInWithOAuth error 발생 시 /login으로 302 redirect', async () => {
		mockSignInWithOAuth.mockResolvedValue({
			data: { url: null },
			error: { message: 'OAuth error' }
		});

		await expect(
			actions.appleOAuth({
				// @ts-expect-error
				request: makeRequest(),
				url: new URL('https://example.com/login'),
				locals: makeLocals()
			})
		).rejects.toMatchObject({ status: 302, location: '/login' });
	});

	it('redirectTo는 origin + /auth/callback 조합', async () => {
		mockSignInWithOAuth.mockResolvedValue({
			data: { url: 'https://apple.com/auth' },
			error: null
		});

		await expect(
			actions.appleOAuth({
				// @ts-expect-error
				request: makeRequest('https://myapp.com'),
				url: new URL('https://myapp.com/login'),
				locals: makeLocals()
			})
		).rejects.toMatchObject({ status: 302 });

		expect(mockSignInWithOAuth).toHaveBeenCalledWith({
			provider: 'apple',
			options: {
				redirectTo: 'https://myapp.com/auth/callback',
				scopes: 'email name'
			}
		});
	});
});
