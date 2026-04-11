// GET /api/auth/token 단위 테스트.
// safeGetSession mock으로 세션 유무 시나리오 검증.

import { describe, it, expect, vi } from 'vitest';
import { GET } from './+server';

function makeEvent(session: { access_token: string } | null) {
	return {
		locals: {
			safeGetSession: vi.fn().mockResolvedValue({ session, user: session ? { id: 'u1' } : null })
		}
	};
}

describe('GET /api/auth/token', () => {
	it('유효한 세션이 있으면 200 + access_token 반환', async () => {
		const event = makeEvent({ access_token: 'fresh.jwt.token' });
		const response = await GET(event as Parameters<typeof GET>[0]);
		const data = await response.json();

		expect(response.status).toBe(200);
		expect(data.token).toBe('fresh.jwt.token');
	});

	it('세션이 없으면 401 + token: null 반환', async () => {
		const event = makeEvent(null);
		const response = await GET(event as Parameters<typeof GET>[0]);
		const data = await response.json();

		expect(response.status).toBe(401);
		expect(data.token).toBeNull();
	});
});
