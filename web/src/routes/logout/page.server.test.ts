// routes/logout/+page.server.ts form action 단위 테스트.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { actions, load } from './+page.server';

const mockSignOut = vi.fn();

function makeLocals() {
	return {
		supabase: {
			auth: { signOut: mockSignOut }
		}
	};
}

beforeEach(() => {
	vi.clearAllMocks();
});

describe('load', () => {
	it('GET 진입 시 항상 /login으로 redirect', async () => {
		// @ts-expect-error - 최소 event 형태
		await expect(load({})).rejects.toMatchObject({
			status: 303,
			location: '/login'
		});
	});
});

describe('actions.default', () => {
	it('supabase.auth.signOut 호출 후 /login으로 redirect', async () => {
		mockSignOut.mockResolvedValue({ error: null });

		await expect(
			// @ts-expect-error
			actions.default({ locals: makeLocals() })
		).rejects.toMatchObject({ status: 303, location: '/login' });

		expect(mockSignOut).toHaveBeenCalled();
	});
});
