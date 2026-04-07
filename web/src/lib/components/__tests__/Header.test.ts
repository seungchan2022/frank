import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';

vi.mock('$lib/stores/auth.svelte', () => ({
	getAuth: () => ({
		user: { email: 'test@example.com' }
	})
}));

vi.mock('$app/forms', () => ({
	enhance: () => ({ destroy: () => undefined })
}));

vi.mock('$app/state', () => ({
	page: {
		url: { pathname: '/feed' }
	}
}));

import Header from '../Header.svelte';

beforeEach(() => {
	vi.clearAllMocks();
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

	it('renders Sign Out button inside form posting to /logout', () => {
		render(Header);

		const signOutBtn = screen.getByRole('button', { name: 'Sign Out' });
		expect(signOutBtn).toBeInTheDocument();
		const form = signOutBtn.closest('form');
		expect(form).not.toBeNull();
		expect(form).toHaveAttribute('method', 'POST');
		expect(form).toHaveAttribute('action', '/logout');
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
