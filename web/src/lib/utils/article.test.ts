import { describe, it, expect } from 'vitest';
import { formatArticleDate, groupArticlesByDate, extractDomain } from './article';
import type { Article } from '$lib/types/article';

function makeArticle(overrides: Partial<Article> = {}): Article {
	return {
		id: '1',
		user_id: 'u1',
		tag_id: null,
		title: 'Test Article',
		url: 'https://example.com/article',
		snippet: null,
		source: 'example.com',
		search_query: null,
		published_at: null,
		created_at: null,
		summary: null,
		insight: null,
		summarized_at: null,
		...overrides
	};
}

describe('formatArticleDate', () => {
	it('returns empty string for null', () => {
		expect(formatArticleDate(null)).toBe('');
	});

	it('returns empty string for invalid date', () => {
		expect(formatArticleDate('not-a-date')).toBe('');
	});

	it('formats a valid ISO date string', () => {
		const result = formatArticleDate('2026-04-04T12:00:00Z');
		expect(result).toBeTruthy();
		// Should contain year 2026
		expect(result).toContain('2026');
	});

	it('formats date-only string', () => {
		const result = formatArticleDate('2026-01-15');
		expect(result).toBeTruthy();
		expect(result).toContain('2026');
	});
});

describe('groupArticlesByDate', () => {
	it('returns empty object for empty array', () => {
		expect(groupArticlesByDate([])).toEqual({});
	});

	it('groups articles by date portion of created_at', () => {
		const articles: Article[] = [
			makeArticle({ id: '1', created_at: '2026-04-04T10:00:00Z' }),
			makeArticle({ id: '2', created_at: '2026-04-04T15:00:00Z' }),
			makeArticle({ id: '3', created_at: '2026-04-03T08:00:00Z' })
		];

		const grouped = groupArticlesByDate(articles);
		expect(Object.keys(grouped)).toHaveLength(2);
		expect(grouped['2026-04-04']).toHaveLength(2);
		expect(grouped['2026-04-03']).toHaveLength(1);
	});

	it('puts articles without created_at under Unknown', () => {
		const articles: Article[] = [
			makeArticle({ id: '1', created_at: null }),
			makeArticle({ id: '2', created_at: '2026-04-04T10:00:00Z' })
		];

		const grouped = groupArticlesByDate(articles);
		expect(grouped['Unknown']).toHaveLength(1);
		expect(grouped['2026-04-04']).toHaveLength(1);
	});
});

describe('extractDomain', () => {
	it('extracts domain from a full URL', () => {
		expect(extractDomain('https://www.example.com/path')).toBe('example.com');
	});

	it('strips www prefix', () => {
		expect(extractDomain('https://www.nytimes.com/article')).toBe('nytimes.com');
	});

	it('keeps subdomain other than www', () => {
		expect(extractDomain('https://blog.example.com')).toBe('blog.example.com');
	});

	it('returns original string for invalid URL', () => {
		expect(extractDomain('not-a-url')).toBe('not-a-url');
	});

	it('handles URL without www', () => {
		expect(extractDomain('https://example.com')).toBe('example.com');
	});
});
