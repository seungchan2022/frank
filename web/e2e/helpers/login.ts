import { expect, type Page } from '@playwright/test';

/**
 * E2E 공유 헬퍼: 이메일 로그인
 *
 * 사전 조건: BASE_URL=http://localhost:5173, 서버 기동 완료
 * 환경변수: TEST_EMAIL, TEST_PASSWORD (.env에서 설정)
 */
export async function login(page: Page): Promise<void> {
	const email = process.env.TEST_EMAIL ?? '';
	const password = process.env.TEST_PASSWORD ?? '';
	await page.goto('/login');
	// Svelte hydration 완료 대기 (value={} 단방향 바인딩 재평가 방지)
	await page.waitForLoadState('networkidle');
	await page.locator('#email').fill(email);
	await page.locator('#password').fill(password);
	// hydration이 fill 값을 덮어쓰지 않는지 확인
	await expect(page.locator('#email')).toHaveValue(email);
	await page.click('button[type="submit"]');
	// 피드 진입 확인
	await expect(page.getByText('전체')).toBeVisible({ timeout: 15_000 });
}
