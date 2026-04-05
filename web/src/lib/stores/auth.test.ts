import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockGetSession = vi.fn();
const mockSignInWithPassword = vi.fn();
const mockSignUp = vi.fn();
const mockSignOut = vi.fn();
const mockOnAuthStateChange = vi.fn();
const mockUnsubscribe = vi.fn();

vi.mock('$lib/supabase', () => ({
	supabase: {
		auth: {
			getSession: () => mockGetSession(),
			signInWithPassword: (...args: unknown[]) => mockSignInWithPassword(...args),
			signUp: (...args: unknown[]) => mockSignUp(...args),
			signOut: () => mockSignOut(),
			onAuthStateChange: (...args: unknown[]) => mockOnAuthStateChange(...args)
		}
	}
}));

import { getAuth, initAuth, cleanupAuth, signInWithEmail, signUpWithEmail, signOut } from './auth.svelte';

beforeEach(() => {
	vi.clearAllMocks();
	mockUnsubscribe.mockClear();
	// Reset module state between tests
	cleanupAuth();
});

describe('getAuth', () => {
	it('returns initial state', () => {
		const auth = getAuth();
		expect(auth.session).toBeNull();
		expect(auth.user).toBeNull();
		expect(auth.isAuthenticated).toBe(false);
	});
});

describe('initAuth', () => {
	it('initializes with existing session', async () => {
		const session = { user: { id: 'u1', email: 'test@test.com' } };
		mockGetSession.mockResolvedValue({ data: { session } });
		mockOnAuthStateChange.mockReturnValue({
			data: { subscription: { unsubscribe: mockUnsubscribe } }
		});

		await initAuth();

		const auth = getAuth();
		expect(auth.session).toEqual(session);
		expect(auth.user).toEqual(session.user);
		expect(auth.isAuthenticated).toBe(true);
		expect(auth.loading).toBe(false);
	});

	it('updates state on auth state change callback', async () => {
		let authCallback: (event: string, session: unknown) => void = () => {};
		mockGetSession.mockResolvedValue({ data: { session: null } });
		mockOnAuthStateChange.mockImplementation((cb: (event: string, session: unknown) => void) => {
			authCallback = cb;
			return { data: { subscription: { unsubscribe: mockUnsubscribe } } };
		});

		await initAuth();

		// Simulate auth state change
		const newSession = { user: { id: 'u2', email: 'new@test.com' } };
		authCallback('SIGNED_IN', newSession);

		const auth = getAuth();
		expect(auth.session).toEqual(newSession);
		expect(auth.user).toEqual(newSession.user);
	});

	it('clears state on sign out callback', async () => {
		const session = { user: { id: 'u1', email: 'test@test.com' } };
		let authCallback: (event: string, session: unknown) => void = () => {};
		mockGetSession.mockResolvedValue({ data: { session } });
		mockOnAuthStateChange.mockImplementation((cb: (event: string, session: unknown) => void) => {
			authCallback = cb;
			return { data: { subscription: { unsubscribe: mockUnsubscribe } } };
		});

		await initAuth();
		authCallback('SIGNED_OUT', null);

		const auth = getAuth();
		expect(auth.session).toBeNull();
		expect(auth.user).toBeNull();
	});

	it('initializes with no session', async () => {
		mockGetSession.mockResolvedValue({ data: { session: null } });
		mockOnAuthStateChange.mockReturnValue({
			data: { subscription: { unsubscribe: mockUnsubscribe } }
		});

		await initAuth();

		const auth = getAuth();
		expect(auth.session).toBeNull();
		expect(auth.user).toBeNull();
		expect(auth.isAuthenticated).toBe(false);
		expect(auth.loading).toBe(false);
	});

	it('does not call onAuthStateChange twice on duplicate initAuth', async () => {
		mockGetSession.mockResolvedValue({ data: { session: null } });
		mockOnAuthStateChange.mockReturnValue({
			data: { subscription: { unsubscribe: mockUnsubscribe } }
		});

		await initAuth();
		await initAuth(); // second call should be no-op

		expect(mockOnAuthStateChange).toHaveBeenCalledTimes(1);
	});

	it('rolls back initialized on getSession failure', async () => {
		mockGetSession.mockRejectedValue(new Error('Network error'));

		await initAuth();
		const auth = getAuth();
		expect(auth.loading).toBe(false);

		// After failure, should be able to re-init
		mockGetSession.mockResolvedValue({ data: { session: null } });
		mockOnAuthStateChange.mockReturnValue({
			data: { subscription: { unsubscribe: mockUnsubscribe } }
		});

		await initAuth();
		expect(mockOnAuthStateChange).toHaveBeenCalledTimes(1);
	});
});

describe('cleanupAuth', () => {
	it('calls unsubscribe once', async () => {
		const localUnsubscribe = vi.fn();
		mockGetSession.mockResolvedValue({ data: { session: null } });
		mockOnAuthStateChange.mockReturnValue({
			data: { subscription: { unsubscribe: localUnsubscribe } }
		});

		await initAuth();
		cleanupAuth();

		expect(localUnsubscribe).toHaveBeenCalledTimes(1);
	});

	it('is idempotent - no error on duplicate calls', async () => {
		mockGetSession.mockResolvedValue({ data: { session: null } });
		mockOnAuthStateChange.mockReturnValue({
			data: { subscription: { unsubscribe: mockUnsubscribe } }
		});

		await initAuth();
		cleanupAuth();
		cleanupAuth(); // should not throw

		expect(mockUnsubscribe).toHaveBeenCalledTimes(1);
	});

	it('prevents callback from updating state after cleanup', async () => {
		let authCallback: (event: string, session: unknown) => void = () => {};
		mockGetSession.mockResolvedValue({ data: { session: null } });
		mockOnAuthStateChange.mockImplementation((cb: (event: string, session: unknown) => void) => {
			authCallback = cb;
			return { data: { subscription: { unsubscribe: mockUnsubscribe } } };
		});

		await initAuth();
		cleanupAuth();

		// Callback fires after cleanup - should not update state
		const newSession = { user: { id: 'u3', email: 'late@test.com' } };
		authCallback('SIGNED_IN', newSession);

		const auth = getAuth();
		expect(auth.session).toBeNull();
		expect(auth.user).toBeNull();
	});

	it('resets all state', async () => {
		const session = { user: { id: 'u1', email: 'test@test.com' } };
		mockGetSession.mockResolvedValue({ data: { session } });
		mockOnAuthStateChange.mockReturnValue({
			data: { subscription: { unsubscribe: mockUnsubscribe } }
		});

		await initAuth();
		const auth = getAuth();
		expect(auth.isAuthenticated).toBe(true);

		cleanupAuth();
		expect(auth.session).toBeNull();
		expect(auth.user).toBeNull();
		expect(auth.loading).toBe(true);
	});
});

describe('signInWithEmail', () => {
	it('signs in successfully', async () => {
		mockSignInWithPassword.mockResolvedValue({ error: null });

		await signInWithEmail('test@test.com', 'password');
		expect(mockSignInWithPassword).toHaveBeenCalledWith({
			email: 'test@test.com',
			password: 'password'
		});
	});

	it('throws on error', async () => {
		mockSignInWithPassword.mockResolvedValue({ error: new Error('Invalid credentials') });

		await expect(signInWithEmail('bad@test.com', 'wrong')).rejects.toThrow('Invalid credentials');
	});
});

describe('signUpWithEmail', () => {
	it('signs up successfully', async () => {
		mockSignUp.mockResolvedValue({ error: null });

		await signUpWithEmail('new@test.com', 'password');
		expect(mockSignUp).toHaveBeenCalledWith({
			email: 'new@test.com',
			password: 'password'
		});
	});

	it('throws on error', async () => {
		mockSignUp.mockResolvedValue({ error: new Error('Already exists') });

		await expect(signUpWithEmail('dup@test.com', 'pass')).rejects.toThrow('Already exists');
	});
});

describe('signOut', () => {
	it('signs out successfully', async () => {
		mockSignOut.mockResolvedValue({ error: null });

		await signOut();
		expect(mockSignOut).toHaveBeenCalled();
	});

	it('throws on error', async () => {
		mockSignOut.mockResolvedValue({ error: new Error('Network error') });

		await expect(signOut()).rejects.toThrow('Network error');
	});
});
