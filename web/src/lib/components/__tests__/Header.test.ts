import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, cleanup } from '@testing-library/svelte';

const mockSignOut = vi.fn();
const mockGoto = vi.fn();

vi.mock('$lib/stores/auth.svelte', () => ({
	getAuth: () => ({
		user: { email: 'test@example.com' }
	}),
	signOut: () => mockSignOut()
}));

vi.mock('$app/navigation', () => ({
	goto: (...args: unknown[]) => mockGoto(...args)
}));

vi.mock('$app/state', () => ({
	page: {
		url: { pathname: '/feed' }
	}
}));

import Header from '../Header.svelte';

beforeEach(() => {
	vi.clearAllMocks();
	mockSignOut.mockResolvedValue(undefined);
});

afterEach(() => {
	cleanup();
});

describe('Header', () => {
	it('renders logo link pointing to /feed', () => {
		render(Header);

		const logo = screen.getByText('Frank');
		expect(logo).toBeInTheDocument();
		expect(logo.closest('a')).toHaveAttribute('href', '/feed');
	});

	it('renders Feed and Settings navigation links', () => {
		render(Header);

		const feedLink = screen.getByText('Feed');
		expect(feedLink.closest('a')).toHaveAttribute('href', '/feed');

		const settingsLink = screen.getByText('Settings');
		expect(settingsLink.closest('a')).toHaveAttribute('href', '/settings');
	});

	it('displays user email', () => {
		render(Header);

		expect(screen.getByText('test@example.com')).toBeInTheDocument();
	});

	it('renders Sign Out button', () => {
		render(Header);

		expect(screen.getByText('Sign Out')).toBeInTheDocument();
	});

	it('calls signOut and navigates to /login on Sign Out click', async () => {
		render(Header);

		await fireEvent.click(screen.getByText('Sign Out'));

		expect(mockSignOut).toHaveBeenCalledOnce();
		expect(mockGoto).toHaveBeenCalledWith('/login');
	});

	it('applies active style to Feed link when on /feed', () => {
		render(Header);

		const feedLink = screen.getByText('Feed');
		expect(feedLink.className).toContain('text-blue-600');
	});

	it('applies inactive style to Settings link when on /feed', () => {
		render(Header);

		const settingsLink = screen.getByText('Settings');
		expect(settingsLink.className).toContain('text-gray-600');
	});
});
