import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockGetSession = vi.fn();
const mockSignInWithPassword = vi.fn();
const mockSignUp = vi.fn();
const mockSignOut = vi.fn();
const mockOnAuthStateChange = vi.fn();

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

import { getAuth, initAuth, signInWithEmail, signUpWithEmail, signOut } from './auth.svelte';

beforeEach(() => {
	vi.clearAllMocks();
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
			data: { subscription: { unsubscribe: vi.fn() } }
		});

		const subscription = await initAuth();
		expect(subscription).toBeDefined();

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
			return { data: { subscription: { unsubscribe: vi.fn() } } };
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
			return { data: { subscription: { unsubscribe: vi.fn() } } };
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
			data: { subscription: { unsubscribe: vi.fn() } }
		});

		await initAuth();

		const auth = getAuth();
		expect(auth.session).toBeNull();
		expect(auth.user).toBeNull();
		expect(auth.isAuthenticated).toBe(false);
		expect(auth.loading).toBe(false);
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
