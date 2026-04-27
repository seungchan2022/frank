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

/**
 * MVP12 M2: BUG-F — 오답 탭용 태그 칩 목록.
 * 실제 오답이 속한 태그 ID만 추출해 칩으로 노출 (filterTags 재사용 방지).
 */
export function buildWrongAnswerFilterTags(
	allTags: Tag[],
	wrongAnswers: WrongAnswer[],
	tagMap: Record<string, string>
): Tag[] {
	const tagIds = new Set(
		wrongAnswers
			.map((wa) => tagMap[wa.articleUrl])
			.filter((id): id is string => id !== undefined)
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
 * MVP12 M2: BUG-F 정책 수정.
 * - selectedTagId=null|'' → 전체 반환 (조기 리턴, undefined 포함 모든 falsy)
 * - selectedTagId가 구체적 태그 ID인 경우: 해당 태그 ID와 정확히 일치하는 항목만 반환
 * - tagId=undefined(즐겨찾기 해제 기사): 태그 선택 시 제외 (기존: 포함 → 수정: 제외)
 *   → tagMap[wa.articleUrl] === undefined 이면 tagId === selectedTagId는 false → 필터 제외됨
 */
export function filterWrongAnswers(
	wrongAnswers: WrongAnswer[],
	tagMap: Record<string, string>,
	selectedTagId: string | null
): WrongAnswer[] {
	if (!selectedTagId) return wrongAnswers;
	return wrongAnswers.filter((wa) => {
		const tagId = tagMap[wa.articleUrl];
		return tagId === selectedTagId;
	});
}
