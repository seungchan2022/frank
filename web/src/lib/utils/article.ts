import type { Article } from '$lib/types/article';

/**
 * Format an article's date for display.
 * Returns a locale-friendly date string, or empty string if no date.
 */
export function formatArticleDate(dateStr: string | null): string {
	if (!dateStr) return '';
	const date = new Date(dateStr);
	if (isNaN(date.getTime())) return '';
	return date.toLocaleDateString('ko-KR', {
		year: 'numeric',
		month: 'short',
		day: 'numeric'
	});
}

/**
 * Group articles by date (YYYY-MM-DD based on created_at).
 * Articles without created_at go under 'Unknown'.
 */
export function groupArticlesByDate(articles: Article[]): Record<string, Article[]> {
	const groups: Record<string, Article[]> = {};
	for (const article of articles) {
		const key = article.created_at ? article.created_at.slice(0, 10) : 'Unknown';
		if (!groups[key]) {
			groups[key] = [];
		}
		groups[key] = [...groups[key], article];
	}
	return groups;
}

/**
 * Extract the domain from a URL for display purposes.
 */
export function extractDomain(url: string): string {
	try {
		const parsed = new URL(url);
		return parsed.hostname.replace(/^www\./, '');
	} catch {
		return url;
	}
}
