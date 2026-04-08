// routes/auth/callback/+server.ts GET 핸들러 단위 테스트.
// Apple OAuth 콜백 처리: code → session 교환 → 리다이렉트.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { GET } from './+server';

const mockExchangeCodeForSession = vi.fn();

function makeLocals() {
	return {
		session: null,
		user: null,
		safeGetSession: vi.fn(),
		supabase: {
			auth: {
				exchangeCodeForSession: mockExchangeCodeForSession
			}
		}
	};
}

function makeEvent(searchParams: Record<string, string>) {
	const url = new URL('https://example.com/auth/callback');
	for (const [k, v] of Object.entries(searchParams)) {
		url.searchParams.set(k, v);
	}
	return {
		url,
		locals: makeLocals()
	};
}

beforeEach(() => {
	vi.clearAllMocks();
});

describe('GET /auth/callback', () => {
	it('code 있고 교환 성공 → / 로 303 redirect', async () => {
		mockExchangeCodeForSession.mockResolvedValue({ error: null });

		await expect(
			GET(
				// @ts-expect-error - 최소 형태
				makeEvent({ code: 'valid-code' })
			)
		).rejects.toMatchObject({ status: 303, location: '/' });

		expect(mockExchangeCodeForSession).toHaveBeenCalledWith('valid-code');
	});

	it('code 있지만 교환 실패 → /login 으로 303 redirect', async () => {
		mockExchangeCodeForSession.mockResolvedValue({
			error: { message: 'Invalid authorization code' }
		});

		await expect(
			GET(
				// @ts-expect-error
				makeEvent({ code: 'bad-code' })
			)
		).rejects.toMatchObject({ status: 303, location: '/login' });
	});

	it('code 없을 때 → /login 으로 303 redirect', async () => {
		await expect(
			GET(
				// @ts-expect-error
				makeEvent({})
			)
		).rejects.toMatchObject({ status: 303, location: '/login' });

		expect(mockExchangeCodeForSession).not.toHaveBeenCalled();
	});
});
