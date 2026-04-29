import { test, expect } from '@playwright/test';

/**
 * Smoke 테스트 (더미)
 *
 * 목적: Playwright 실행 환경 검증 (E2E 인프라 확인)
 * 실제 시나리오는 M4에서 추가
 *
 * 주의: 백엔드 연결 없이 실행 가능 (about:blank 사용)
 */
test('smoke: Playwright 실행 환경 동작 확인', async ({ page }) => {
	await page.goto('about:blank');
	// 실행 환경 자체가 정상 동작하는지 검증
	expect(1).toBe(1);
});
