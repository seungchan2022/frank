import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [sveltekit()],
	resolve: {
		conditions: ['browser']
	},
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}'],
		environment: 'jsdom',
		setupFiles: ['src/test-utils/setup.ts'],
		coverage: {
			provider: 'v8',
			include: ['src/lib/**/*.ts'],
			exclude: ['src/lib/**/*.test.ts', 'src/lib/**/*.spec.ts', 'src/lib/supabase.ts', 'src/lib/server/**', 'src/lib/index.ts', 'src/lib/types/**'],
			thresholds: {
				statements: 90,
				branches: 90,
				functions: 90,
				lines: 90
			}
		}
	}
});
