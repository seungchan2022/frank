import { expect, type Page } from '@playwright/test';

/**
 * E2E 공유 헬퍼: 이메일 로그인 (test@test.com / Test1234!)
 *
 * 사전 조건: BASE_URL=http://localhost:5173, 서버 기동 완료
 */
export async function login(page: Page): Promise<void> {
	await page.goto('/login');
	// Svelte hydration 완료 대기 (value={} 단방향 바인딩 재평가 방지)
	await page.waitForLoadState('networkidle');
	await page.locator('#email').fill('test@test.com');
	await page.locator('#password').fill('Test1234!');
	// hydration이 fill 값을 덮어쓰지 않는지 확인
	await expect(page.locator('#email')).toHaveValue('test@test.com');
	await page.click('button[type="submit"]');
	// 피드 진입 확인
	await expect(page.getByText('전체')).toBeVisible({ timeout: 15_000 });
}
