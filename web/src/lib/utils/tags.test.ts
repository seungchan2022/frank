import { describe, it, expect } from 'vitest';
import { groupByCategory, validateTagSelection } from './tags';
import type { Tag } from '$lib/types/tag';

describe('groupByCategory', () => {
	it('groups tags by category', () => {
		const tags: Tag[] = [
			{ id: '1', name: 'AI/ML', category: 'Tech' },
			{ id: '2', name: 'Web', category: 'Tech' },
			{ id: '3', name: 'Startup', category: 'Business' }
		];

		const grouped = groupByCategory(tags);
		expect(Object.keys(grouped)).toEqual(['Tech', 'Business']);
		expect(grouped['Tech']).toHaveLength(2);
		expect(grouped['Business']).toHaveLength(1);
	});

	it('uses "Other" for null category', () => {
		const tags: Tag[] = [{ id: '1', name: 'Misc', category: null }];

		const grouped = groupByCategory(tags);
		expect(grouped['Other']).toHaveLength(1);
	});

	it('returns empty object for empty array', () => {
		expect(groupByCategory([])).toEqual({});
	});
});

describe('validateTagSelection', () => {
	it('returns null when enough tags selected', () => {
		expect(validateTagSelection(['1', '2'])).toBeNull();
	});

	it('returns error when no tags selected', () => {
		expect(validateTagSelection([])).toBe('Please select at least 1 tag.');
	});

	it('respects custom minCount', () => {
		expect(validateTagSelection(['1'], 3)).toBe('Please select at least 3 tags.');
		expect(validateTagSelection(['1', '2', '3'], 3)).toBeNull();
	});
});
