import { defineConfig } from '@playwright/test';

/**
 * Playwright E2E 설정
 *
 * 실행 전 필수: scripts/deploy.sh --target=api,front --native 로 서버 선행 기동
 * baseURL: BASE_URL 환경변수 (미설정 시 localhost:5173 fallback)
 *
 * 참고: webServer 자동화는 M4 부채로 이관 (BUG/DEBT 문서 참조)
 */
export default defineConfig({
	testDir: './e2e',
	// 기존 Vitest 테스트(src/**)와 경로가 겹치지 않음 — 별도 디렉토리 분리
	timeout: 30_000,
	retries: 0,
	workers: 1,
	reporter: 'list',
	use: {
		baseURL: process.env.BASE_URL ?? 'http://localhost:5173',
		headless: true,
		screenshot: 'only-on-failure',
		video: 'off',
	},
	projects: [
		{
			name: 'chromium',
			use: { browserName: 'chromium' },
		},
	],
});
