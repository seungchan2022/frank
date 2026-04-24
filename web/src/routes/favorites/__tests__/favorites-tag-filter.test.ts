// MVP11 M3: favorites 태그 칩 필터 로직 단위 테스트

import { describe, it, expect } from 'vitest';
import type { Favorite } from '$lib/types/favorite';
import type { WrongAnswer } from '$lib/types/quiz';
import type { Tag } from '$lib/types/tag';
import {
	buildFavTagIds,
	buildFilterTags,
	buildWrongAnswerTagMap,
	filterFavorites,
	filterWrongAnswers
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

const makeWrongAnswer = (id: string, articleUrl: string): WrongAnswer => ({
	id,
	userId: 'user-1',
	articleUrl,
	articleTitle: `기사 ${articleUrl}`,
	question: '질문',
	options: ['A', 'B', 'C', 'D'],
	correctIndex: 0,
	userIndex: 1,
	explanation: null,
	createdAt: '2026-04-24T00:00:00Z'
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

	describe('오답 탭 필터링', () => {
		const favorites = [
			makeFavorite('https://a.com', 'tag-ai'),
			makeFavorite('https://b.com', 'tag-economy')
		];
		const tagMap = buildWrongAnswerTagMap(favorites);

		it('selectedTagId=null이면 전체 반환', () => {
			const wrongAnswers = [
				makeWrongAnswer('wa-1', 'https://a.com'),
				makeWrongAnswer('wa-2', 'https://b.com')
			];
			expect(filterWrongAnswers(wrongAnswers, tagMap, null)).toHaveLength(2);
		});

		it('selectedTagId 선택 시 해당 태그 오답만 반환', () => {
			const wrongAnswers = [
				makeWrongAnswer('wa-1', 'https://a.com'), // tag-ai
				makeWrongAnswer('wa-2', 'https://b.com') // tag-economy
			];
			const result = filterWrongAnswers(wrongAnswers, tagMap, 'tag-ai');
			expect(result).toHaveLength(1);
			expect(result[0].id).toBe('wa-1');
		});

		it('즐겨찾기 해제된 기사(태그 미매핑) 오답은 태그 필터 적용 시에도 항상 포함', () => {
			const wrongAnswers = [
				makeWrongAnswer('wa-1', 'https://a.com'), // tag-ai (매핑 있음)
				makeWrongAnswer('wa-2', 'https://removed.com') // 즐겨찾기 해제됨 (매핑 없음)
			];
			const result = filterWrongAnswers(wrongAnswers, tagMap, 'tag-ai');
			expect(result).toHaveLength(2);
			expect(result.map((wa) => wa.id)).toContain('wa-2');
		});

		it('태그 매핑 없는 오답만 있을 때 태그 필터 적용 시 전체 반환', () => {
			const wrongAnswers = [makeWrongAnswer('wa-1', 'https://removed.com')];
			const result = filterWrongAnswers(wrongAnswers, tagMap, 'tag-economy');
			expect(result).toHaveLength(1);
		});
	});
});
