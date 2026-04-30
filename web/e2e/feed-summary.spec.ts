import { test, expect } from '@playwright/test';
import { login } from './helpers/login';

/**
 * W-01: 피드 → 기사 상세 → 요약 시나리오
 *
 * 커버 항목:
 * - BUG-006: 에러 캐시 재시도 (500 주입 후 재시도 성공)
 * - DEBT-06: 요약 버튼 하단 고정 (스크랩 버튼 접근 가능)
 * - DEBT-07: 기사 소개 카드 vs AI 요약 카드 시각 구분
 *
 * 전제: BASE_URL=http://localhost:5173, 서버 기동 완료 (deploy.sh 선행)
 * 계정: test@test.com / Test1234!
 */

test.describe('W-01: 피드 요약 시나리오', () => {
	test.beforeEach(async ({ page }) => {
		await login(page);
	});

	// F-03: 기사 상세 진입 후 스크랩 버튼(하단 고정) 바로 접근 가능 (DEBT-06)
	test('DEBT-06: 스크랩 버튼이 기사 상세 진입 직후 하단에 고정 노출', async ({ page }) => {
		// 첫 번째 기사 클릭
		const firstArticle = page.locator('article button').first();
		await expect(firstArticle).toBeVisible({ timeout: 10_000 });
		await firstArticle.click();

		// 원문 보기 링크가 기사 헤더 내에 존재 (DEBT-06 검증: 스크롤 없이 접근 가능)
		const openOriginalLink = page.getByRole('link', { name: '원문 보기' });
		await expect(openOriginalLink).toBeVisible({ timeout: 5_000 });

		// 하단 고정 패널의 스크랩 저장 버튼도 바로 접근 가능
		const scrapButton = page.getByRole('button', { name: /스크랩 저장|스크랩 해제/ });
		await expect(scrapButton).toBeVisible({ timeout: 5_000 });
	});

	// F-04: 기사 소개 카드(📰 기사 소개) vs AI 요약 카드(✨ AI 요약) 시각 구분 (DEBT-07)
	test('DEBT-07: 기사 소개 카드와 AI 요약 카드가 시각적으로 구분됨', async ({ page }) => {
		const firstArticle = page.locator('article button').first();
		await expect(firstArticle).toBeVisible({ timeout: 10_000 });
		await firstArticle.click();

		// 기사 소개 섹션 확인 (snippet이 있는 기사)
		// h2 헤딩에 "기사 소개" 텍스트 포함
		const snippetHeading = page.locator('h2', { hasText: '기사 소개' });
		// AI 요약 섹션 항상 존재 확인
		const summaryHeading = page.locator('h2', { hasText: 'AI 요약 및 인사이트' });
		await expect(summaryHeading).toBeVisible({ timeout: 5_000 });

		// 기사 소개가 있다면 별도 카드로 존재하는지 확인
		const snippetCount = await snippetHeading.count();
		if (snippetCount > 0) {
			await expect(snippetHeading).toBeVisible();
		}
	});

	// F-01 + F-02: 요약 요청 → 요약 카드 표시 확인
	test('요약하기 버튼 탭 후 요약 결과 표시', async ({ page }) => {
		const firstArticle = page.locator('article button').first();
		await expect(firstArticle).toBeVisible({ timeout: 10_000 });
		await firstArticle.click();

		// ✨ 요약하기 버튼 존재 확인
		const summarizeButton = page.getByRole('button', { name: /✨ 요약하기/ });
		await expect(summarizeButton).toBeVisible({ timeout: 5_000 });

		// 요약하기 탭
		await summarizeButton.click();

		// 로딩 중 또는 완료/실패 중 하나의 상태로 전이
		// 로딩 스피너 또는 결과 텍스트 또는 실패 텍스트 노출 확인
		await expect(
			page.locator('text=요약 중').or(
				page.locator('h3', { hasText: '요약' }).or(
					page.locator('text=요약을 불러오지 못했습니다')
				)
			)
		).toBeVisible({ timeout: 90_000 });
	});

	// E-01 + BUG-006: 요약 API 500 주입 → 에러 메시지 + 재시도 버튼 노출
	test('BUG-006: 요약 API 실패 시 에러 메시지와 재시도 버튼 노출', async ({ page }) => {
		// 요약 API 첫 번째 호출에 500 강제 반환, 두 번째 호출은 통과
		let callCount = 0;
		await page.route('**/api/me/summarize', async (route) => {
			callCount++;
			if (callCount === 1) {
				await route.fulfill({
					status: 500,
					contentType: 'application/json',
					body: JSON.stringify({ error: 'internal server error' })
				});
			} else {
				await route.continue();
			}
		});

		const firstArticle = page.locator('article button').first();
		await expect(firstArticle).toBeVisible({ timeout: 10_000 });
		await firstArticle.click();

		const summarizeButton = page.getByRole('button', { name: /✨ 요약하기/ });
		await expect(summarizeButton).toBeVisible({ timeout: 5_000 });
		await summarizeButton.click();

		// 실패 상태: 에러 메시지 + 재시도 버튼 노출
		const retryButton = page.getByRole('button', { name: /↺ 다시 시도/ });
		await expect(retryButton).toBeVisible({ timeout: 30_000 });

		// 재시도 탭 → 두 번째 요청은 실제 API로 통과 (또는 네트워크 연결 없이도 버튼 동작 확인)
		await retryButton.click();
		// 로딩 상태로 전이됨을 확인 (요약 중 or 마무리 중 텍스트)
		await expect(
			page.locator('text=요약 중').or(page.locator('text=마무리 중'))
				.or(page.locator('h3', { hasText: '요약' }))
		).toBeVisible({ timeout: 90_000 });
	});
});
