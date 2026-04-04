import type { Tag } from '$lib/types/tag';

export function groupByCategory(tags: Tag[]): Record<string, Tag[]> {
	return tags.reduce<Record<string, Tag[]>>((acc, tag) => {
		const cat = tag.category ?? 'Other';
		if (!acc[cat]) acc[cat] = [];
		acc[cat].push(tag);
		return acc;
	}, {});
}

export function validateTagSelection(selectedIds: string[], minCount = 1): string | null {
	if (selectedIds.length < minCount) {
		return `Please select at least ${minCount} tag${minCount > 1 ? 's' : ''}.`;
	}
	return null;
}
