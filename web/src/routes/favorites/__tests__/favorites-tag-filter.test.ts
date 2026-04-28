// MVP11 M3: favorites 태그 칩 필터 로직 단위 테스트
// MVP12 M2: BUG-C(selectedTagId 초기화) + BUG-F(filterWrongAnswers 정책 수정) 테스트 추가
// MVP13 M2: favorites 브릿지(tagMap) 제거 — WrongAnswer.tagId 직접 사용 테스트

import { describe, it, expect } from 'vitest';
import type { Favorite } from '$lib/types/favorite';
import type { WrongAnswer } from '$lib/types/quiz';
import type { Tag } from '$lib/types/tag';
import {
	buildFavTagIds,
	buildFilterTags,
	buildWrongAnswerFilterTags,
	filterFavorites,
	filterWrongAnswers,
	shouldResetTagId
} from '$lib/utils/favorites-filter';

const tags: Tag[] = [
	{ id: 'tag-ai', name: 'AI', category: '기술' },
	{ id: 'tag-economy', name: '경제', category: '사회' },
	{ id: 'tag-health', name: '건강', category: '생활' }
];

const makeFavorite = (url: string, tagId: string | null): Favorite => ({
	id: `fav-${url}`,
	userId: 'user-1',
	title: `기사 ${url}`,
	url,
	snippet: null,
	source: 'test',
	publishedAt: null,
	tagId,
	summary: null,
	insight: null,
	likedAt: null,
	createdAt: '2026-04-24T00:00:00Z'
});

// MVP13 M2: tagId 직접 포함
const makeWrongAnswer = (id: string, articleUrl: string, tagId: string | null = null): WrongAnswer => ({
	id,
	userId: 'user-1',
	articleUrl,
	articleTitle: `기사 ${articleUrl}`,
	question: '질문',
	options: ['A', 'B', 'C', 'D'],
	correctIndex: 0,
	userIndex: 1,
	explanation: null,
	createdAt: '2026-04-24T00:00:00Z',
	tagId
});

// --- 테스트 ---

describe('favorites 태그 칩 필터 (MVP11 M3)', () => {
	describe('filterTags 목록 구성', () => {
		it('즐겨찾기에 있는 tagId만 칩에 표시', () => {
			const favorites = [
				makeFavorite('https://a.com', 'tag-ai'),
				makeFavorite('https://b.com', 'tag-economy'),
				makeFavorite('https://c.com', null)
			];
			const favTagIds = buildFavTagIds(favorites);
			const result = buildFilterTags(tags, favTagIds);
			expect(result.map((t) => t.id)).toEqual(['tag-ai', 'tag-economy']);
		});

		it('즐겨찾기가 전부 tagId=null이면 칩 없음', () => {
			const favorites = [makeFavorite('https://a.com', null)];
			const favTagIds = buildFavTagIds(favorites);
			const result = buildFilterTags(tags, favTagIds);
			expect(result).toHaveLength(0);
		});

		it('즐겨찾기가 비어있으면 칩 없음', () => {
			const favTagIds = buildFavTagIds([]);
			const result = buildFilterTags(tags, favTagIds);
			expect(result).toHaveLength(0);
		});
	});

	describe('기사 탭 필터링', () => {
		it('selectedTagId=null이면 전체 반환', () => {
			const favorites = [
				makeFavorite('https://a.com', 'tag-ai'),
				makeFavorite('https://b.com', 'tag-economy')
			];
			expect(filterFavorites(favorites, null)).toHaveLength(2);
		});

		it('selectedTagId 선택 시 해당 태그 기사만 반환', () => {
			const favorites = [
				makeFavorite('https://a.com', 'tag-ai'),
				makeFavorite('https://b.com', 'tag-economy'),
				makeFavorite('https://c.com', 'tag-ai')
			];
			const result = filterFavorites(favorites, 'tag-ai');
			expect(result).toHaveLength(2);
			expect(result.every((f) => f.tagId === 'tag-ai')).toBe(true);
		});

		it('결과 없을 때 빈 배열 반환', () => {
			const favorites = [makeFavorite('https://a.com', 'tag-economy')];
			expect(filterFavorites(favorites, 'tag-ai')).toHaveLength(0);
		});
	});

	// MVP13 M2: filterWrongAnswers — WrongAnswer.tagId 직접 비교 (favorites 브릿지 제거)
	describe('오답 탭 필터링 (MVP13 M2)', () => {
		it('selectedTagId=null이면 전체 반환', () => {
			const wrongAnswers = [
				makeWrongAnswer('wa-1', 'https://a.com', 'tag-ai'),
				makeWrongAnswer('wa-2', 'https://b.com', 'tag-economy')
			];
			expect(filterWrongAnswers(wrongAnswers, null)).toHaveLength(2);
		});

		it('selectedTagId 선택 시 해당 tagId 오답만 반환', () => {
			const wrongAnswers = [
				makeWrongAnswer('wa-1', 'https://a.com', 'tag-ai'),
				makeWrongAnswer('wa-2', 'https://b.com', 'tag-economy')
			];
			const result = filterWrongAnswers(wrongAnswers, 'tag-ai');
			expect(result).toHaveLength(1);
			expect(result[0].id).toBe('wa-1');
		});

		it('tagId=null 오답은 태그 선택 시 제외', () => {
			const wrongAnswers = [
				makeWrongAnswer('wa-1', 'https://a.com', 'tag-ai'),
				makeWrongAnswer('wa-2', 'https://b.com', null) // tagId 없음
			];
			const result = filterWrongAnswers(wrongAnswers, 'tag-ai');
			expect(result).toHaveLength(1);
			expect(result[0].id).toBe('wa-1');
		});

		it('tagId=null 오답만 있을 때 태그 필터 적용 시 빈 배열 반환', () => {
			const wrongAnswers = [makeWrongAnswer('wa-1', 'https://a.com', null)];
			const result = filterWrongAnswers(wrongAnswers, 'tag-economy');
			expect(result).toHaveLength(0);
		});

		it('선택된 태그에 해당하는 오답 없으면 빈 배열', () => {
			const wrongAnswers = [makeWrongAnswer('wa-1', 'https://a.com', 'tag-ai')];
			const result = filterWrongAnswers(wrongAnswers, 'tag-economy');
			expect(result).toHaveLength(0);
		});

		it('selectedTagId=null → tagId=null 오답도 전체에 포함', () => {
			const wrongAnswers = [
				makeWrongAnswer('wa-1', 'https://a.com', 'tag-ai'),
				makeWrongAnswer('wa-2', 'https://b.com', null)
			];
			const result = filterWrongAnswers(wrongAnswers, null);
			expect(result).toHaveLength(2);
		});
	});

	// MVP13 M2: buildWrongAnswerFilterTags — WrongAnswer.tagId 직접 집계 (favorites 브릿지 제거)
	describe('wrongAnswerFilterTags 구성 (MVP13 M2)', () => {
		it('실제 오답의 tagId에 해당하는 태그만 칩으로 노출', () => {
			const wrongAnswers = [
				makeWrongAnswer('wa-1', 'https://a.com', 'tag-ai') // tag-ai만 포함
			];
			const result = buildWrongAnswerFilterTags(tags, wrongAnswers);
			expect(result.map((t) => t.id)).toEqual(['tag-ai']);
			expect(result.some((t) => t.id === 'tag-economy')).toBe(false);
		});

		it('wrongAnswers 빈 배열이면 태그 칩 없음', () => {
			const result = buildWrongAnswerFilterTags(tags, []);
			expect(result).toHaveLength(0);
		});

		it('tagId=null 오답만 있으면 태그 칩 없음', () => {
			const wrongAnswers = [makeWrongAnswer('wa-1', 'https://a.com', null)];
			const result = buildWrongAnswerFilterTags(tags, wrongAnswers);
			expect(result).toHaveLength(0);
		});

		it('여러 태그 오답 → 해당 태그들만 칩으로 노출', () => {
			const wrongAnswers = [
				makeWrongAnswer('wa-1', 'https://a.com', 'tag-ai'),
				makeWrongAnswer('wa-2', 'https://b.com', 'tag-economy'),
				makeWrongAnswer('wa-3', 'https://c.com', 'tag-ai') // 중복 태그
			];
			const result = buildWrongAnswerFilterTags(tags, wrongAnswers);
			// tag-health는 오답 없음 → 제외
			expect(result.map((t) => t.id)).not.toContain('tag-health');
			// tag-ai, tag-economy만 포함
			expect(result.map((t) => t.id)).toContain('tag-ai');
			expect(result.map((t) => t.id)).toContain('tag-economy');
			expect(result).toHaveLength(2);
		});
	});

	// MVP12 M2: BUG-C — shouldResetTagId 회귀 테스트 (T-05)
	// handleRemoveFavorite에서 shouldResetTagId를 사용하므로 이 함수의 정확성이 BUG-C 수정의 핵심.
	describe('BUG-C: shouldResetTagId 회귀 테스트 (MVP12 M2, T-05)', () => {
		it('마지막 tag-ai 즐겨찾기 삭제 후 → shouldResetTagId=true (초기화 필요)', () => {
			// BUG 재현 시나리오: tag-ai 필터 선택 상태에서 유일한 tag-ai 기사 삭제
			const remaining: Favorite[] = [makeFavorite('https://b.com', 'tag-economy')];
			expect(shouldResetTagId(remaining, 'tag-ai')).toBe(true);
		});

		it('같은 태그 다른 기사가 남아있으면 → shouldResetTagId=false (초기화 불필요)', () => {
			// E-01: 같은 태그 기사가 남아있으면 탭을 유지해야 함
			const remaining: Favorite[] = [
				makeFavorite('https://b.com', 'tag-ai'),
				makeFavorite('https://c.com', 'tag-economy')
			];
			expect(shouldResetTagId(remaining, 'tag-ai')).toBe(false);
		});

		it('selectedTagId=null이면 → shouldResetTagId=false (전체 탭은 항상 유지)', () => {
			const remaining: Favorite[] = [];
			expect(shouldResetTagId(remaining, null)).toBe(false);
		});

		it('즐겨찾기 전체 삭제 후 → shouldResetTagId=true', () => {
			expect(shouldResetTagId([], 'tag-ai')).toBe(true);
		});

		it('tagId=null 즐겨찾기만 남아있고 tag-ai 선택 중 → shouldResetTagId=true', () => {
			// tag-ai 태그 없는 항목만 남은 경우
			const remaining: Favorite[] = [makeFavorite('https://b.com', null)];
			expect(shouldResetTagId(remaining, 'tag-ai')).toBe(true);
		});
	});
});
