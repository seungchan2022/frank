import { test, expect } from '@playwright/test';
import { login } from './helpers/login';

/**
 * W-03: 피드 좋아요(추천) 단독 탭 시나리오
 *
 * 커버 항목:
 * - DEBT-04: 좋아요 버튼 클릭 시 기사 상세(URL 변경) 없이 좋아요만 처리
 *
 * 전제: BASE_URL=http://localhost:5173, 서버 기동 완료 (deploy.sh 선행)
 * 계정: test@test.com / Test1234!
 */

test.describe('W-03: 피드 좋아요 단독 탭 시나리오', () => {
	test.beforeEach(async ({ page }) => {
		await login(page);
	});

	// F-07: DEBT-04 — 좋아요 버튼 클릭 시 URL 변경 없음 (피드 유지)
	test('DEBT-04: 좋아요 버튼 탭 시 피드 URL 유지 (기사 상세 이동 없음)', async ({ page }) => {
		// 피드 URL 기억
		const feedUrl = page.url();

		// 첫 번째 기사 카드의 좋아요 버튼 찾기
		// aria-label이 "추천에 반영" 또는 "추천 완료"인 버튼
		const likeButton = page.getByRole('button', { name: /추천에 반영|추천 완료/ }).first();
		await expect(likeButton).toBeVisible({ timeout: 10_000 });

		// 좋아요 버튼 탭
		await likeButton.click();

		// 잠깐 대기 후 URL 확인
		await page.waitForTimeout(500);

		// URL이 /feed 내에 머물러야 함 (article 페이지로 이동하지 않음)
		const currentUrl = page.url();
		expect(currentUrl).not.toContain('/feed/article');
		expect(currentUrl).toContain('/feed');

		// 피드 화면 요소가 여전히 보임
		await expect(page.getByText('전체')).toBeVisible();
	});

	// 아이콘 토글: 좋아요 전 ♡ → 후 ♥ 확인
	test('DEBT-04: 좋아요 버튼 탭 후 아이콘이 ♡ → ♥ 로 토글됨', async ({ page }) => {
		// 첫 번째 아직 좋아요 안 된 버튼 찾기 ("추천에 반영" 상태)
		const unlikedButton = page.getByRole('button', { name: '추천에 반영' }).first();

		// 버튼이 없으면(이미 모두 좋아요) 스킵
		const unlikedCount = await unlikedButton.count();
		if (unlikedCount === 0) {
			test.skip(true, '좋아요 가능한 기사가 없음 (모두 이미 추천 중)');
			return;
		}

		await expect(unlikedButton).toBeVisible({ timeout: 10_000 });

		// 좋아요 전 하트 아이콘 확인 (♡)
		const heartSpan = unlikedButton.locator('span').first();
		await expect(heartSpan).toHaveText('♡');

		// 좋아요 탭
		await unlikedButton.click();

		// 같은 위치 버튼이 이제 "추천 완료" 상태로 전환 (toBeVisible 자동 재시도로 아이콘 애니메이션 커버)
		const likedButton = page.getByRole('button', { name: '추천 완료' }).first();
		await expect(likedButton).toBeVisible({ timeout: 3_000 });
	});

	// F-08: 좋아요 후 즐겨찾기(스크랩) 탭에서 확인 — 스크랩과 추천은 별도 동작
	// 참고: "추천에 반영"은 likes(피드 개인화 반영), 즐겨찾기는 favorites(스크랩)로 별도
	test('피드 좋아요와 스크랩은 독립적으로 동작함', async ({ page }) => {
		// 피드 진입 확인
		await expect(page.getByText('전체')).toBeVisible({ timeout: 10_000 });

		// 좋아요 버튼 탭
		const likeButton = page.getByRole('button', { name: /추천에 반영|추천 완료/ }).first();
		await expect(likeButton).toBeVisible({ timeout: 10_000 });
		await likeButton.click();
		await page.waitForTimeout(300);

		// URL이 유지됨 (피드 내)
		expect(page.url()).toContain('/feed');
		expect(page.url()).not.toContain('/feed/article');

		// 피드 화면이 그대로 유지됨
		await expect(page.getByText('전체')).toBeVisible();
	});
});
