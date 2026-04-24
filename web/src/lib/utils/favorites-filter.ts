import type { Favorite } from '$lib/types/favorite';
import type { WrongAnswer } from '$lib/types/quiz';
import type { Tag } from '$lib/types/tag';

export function buildFavTagIds(favorites: Favorite[]): Set<string> {
	return new Set(
		favorites.map((f) => f.tagId).filter((id): id is string => id !== null)
	);
}

export function buildFilterTags(allTags: Tag[], favTagIds: Set<string>): Tag[] {
	return allTags.filter((t) => favTagIds.has(t.id));
}

export function buildWrongAnswerTagMap(favorites: Favorite[]): Record<string, string> {
	return Object.fromEntries(
		favorites
			.filter((f) => f.tagId !== null)
			.map((f) => [f.url, f.tagId as string])
	);
}

export function filterFavorites(favorites: Favorite[], selectedTagId: string | null): Favorite[] {
	return selectedTagId ? favorites.filter((f) => f.tagId === selectedTagId) : favorites;
}

export function filterWrongAnswers(
	wrongAnswers: WrongAnswer[],
	tagMap: Record<string, string>,
	selectedTagId: string | null
): WrongAnswer[] {
	if (!selectedTagId) return wrongAnswers;
	return wrongAnswers.filter((wa) => {
		const tagId = tagMap[wa.articleUrl];
		return tagId === undefined || tagId === selectedTagId;
	});
}
