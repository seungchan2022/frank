import { test, expect } from '@playwright/test';
import { login } from './helpers/login';

/**
 * W-02: 태그 탭 전환 시나리오
 *
 * 커버 항목:
 * - BUG-008: 태그 탭 전환 시 기사 목록 깜빡임 없음 (DOM 삭제 후 재삽입 없음)
 *
 * 전제: BASE_URL=http://localhost:5173, 서버 기동 완료 (deploy.sh 선행)
 * 계정: test@test.com / Test1234!
 */

/** MutationObserver 추적용 전역 확장 타입 */
type ObserverWindow = Window & { __removedCount?: number; __obs?: MutationObserver };

test.describe('W-02: 태그 탭 전환 시나리오', () => {
	test.beforeEach(async ({ page }) => {
		await login(page);
	});

	// F-05: 로그인 → 태그 클릭 → 목록 변경 확인
	test('태그 탭 클릭 시 피드 목록이 전환됨', async ({ page }) => {
		// 피드 초기 진입 확인
		await expect(page.getByRole('button', { name: '전체' })).toBeVisible({ timeout: 10_000 });

		// 태그 탭 버튼 목록 확인 (전체 + 각 태그)
		const tagButtons = page.locator('button.rounded-full');
		const tagCount = await tagButtons.count();

		if (tagCount <= 1) {
			// 태그가 없는 경우 테스트 스킵 — 태그 설정 필요
			test.skip(true, '태그가 없어 탭 전환 테스트 불가 (태그 관리에서 태그 추가 필요)');
			return;
		}

		// 전체 탭 이외의 첫 번째 태그 탭 클릭
		const firstTagButton = tagButtons.nth(1);
		const tagName = await firstTagButton.textContent();
		await firstTagButton.click();

		// 선택된 태그 버튼이 활성 스타일(bg-blue-600)을 가짐 확인 (toHaveClass는 자동 재시도로 안정적)
		await expect(firstTagButton).toHaveClass(/bg-blue-600/);

		// 전체 탭으로 복귀
		await page.getByRole('button', { name: '전체' }).click();
		await expect(page.getByRole('button', { name: '전체' })).toHaveClass(/bg-gray-900/);

		console.log(`태그 "${tagName?.trim()}" 탭 전환 확인 완료`);
	});

	// F-06: 탭 전환 중 깜빡임 없음 (BUG-008) — DOM 삭제 후 재삽입 없음 확인
	test('BUG-008: 태그 탭 전환 시 기사 컨테이너가 재마운트되지 않음', async ({ page }) => {
		await expect(page.getByRole('button', { name: '전체' })).toBeVisible({ timeout: 10_000 });

		// 태그 버튼 목록 확인
		const tagButtons = page.locator('button.rounded-full');
		const tagCount = await tagButtons.count();

		if (tagCount <= 1) {
			test.skip(true, '태그가 없어 BUG-008 탭 전환 테스트 불가');
			return;
		}

		// MutationObserver를 클릭 전에 전역으로 설치
		// (evaluate가 Promise로 블록되면 이후 click이 실행되지 않으므로 분리)
		await page.evaluate(() => {
			const w = window as ObserverWindow;
			w.__removedCount = 0;
			const grid = document.querySelector('.grid');
			if (grid) {
				w.__obs = new MutationObserver((mutations) => {
					mutations.forEach((m) => {
						w.__removedCount = (w.__removedCount ?? 0) + m.removedNodes.length;
					});
				});
				w.__obs.observe(grid, { childList: true, subtree: false });
			}
		});

		// 태그 탭 클릭 (전환 유발) — observer 설치 후에 클릭
		const firstTagButton = tagButtons.nth(1);
		await firstTagButton.click();
		await page.waitForTimeout(600); // 전환 완료 대기

		// observer 해제 및 결과 수집
		const removedNodesCount = await page.evaluate(() => {
			const w = window as ObserverWindow;
			w.__obs?.disconnect();
			return w.__removedCount ?? -1;
		});

		// E-02: 태그 선택 후 빈 목록일 때 빈 상태 UI 확인 또는 기사 목록 확인
		const hasArticles = (await page.locator('article').count()) > 0;
		const hasEmptyState = await page.locator('text=No articles yet').isVisible();

		expect(hasArticles || hasEmptyState).toBe(true);

		// BUG-008: grid 컨테이너의 childList removedNodes가 없어야 함 (재마운트 없음)
		// grid가 없어 observer 미설치된 경우(-1)는 건너뜀
		if (removedNodesCount >= 0) {
			expect(removedNodesCount).toBe(0);
		}

		console.log(`BUG-008: 탭 전환 후 DOM removedNodes 수: ${removedNodesCount}`);
	});
});
