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

/**
 * MVP13 M2: BUG-F — 오답 탭용 태그 칩 목록.
 * WrongAnswer.tagId 직접 집계 — favorites 브릿지(tagMap) 제거.
 */
export function buildWrongAnswerFilterTags(
	allTags: Tag[],
	wrongAnswers: WrongAnswer[]
): Tag[] {
	const tagIds = new Set(
		wrongAnswers
			.map((wa) => wa.tagId)
			.filter((id): id is string => id !== null)
	);
	return allTags.filter((t) => tagIds.has(t.id));
}

export function filterFavorites(favorites: Favorite[], selectedTagId: string | null): Favorite[] {
	return selectedTagId ? favorites.filter((f) => f.tagId === selectedTagId) : favorites;
}

/**
 * MVP12 M2: BUG-C — 즐겨찾기 삭제 후 selectedTagId 초기화 여부 판단.
 * 삭제 후 남은 favorites에 현재 selectedTagId 매칭 항목이 없으면 true(초기화 필요).
 */
export function shouldResetTagId(
	remainingFavorites: Favorite[],
	selectedTagId: string | null
): boolean {
	if (selectedTagId === null) return false;
	return !remainingFavorites.some((f) => f.tagId === selectedTagId);
}

/**
 * MVP13 M2: favorites 브릿지(tagMap) 제거 — WrongAnswer.tagId 직접 비교.
 * - selectedTagId=null|'' → 전체 반환
 * - selectedTagId != null: wa.tagId === selectedTagId 인 항목만 반환
 */
export function filterWrongAnswers(
	wrongAnswers: WrongAnswer[],
	selectedTagId: string | null
): WrongAnswer[] {
	if (!selectedTagId) return wrongAnswers;
	return wrongAnswers.filter((wa) => wa.tagId === selectedTagId);
}
