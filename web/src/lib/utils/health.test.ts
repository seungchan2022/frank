import { describe, it, expect } from 'vitest';
import { isHealthy } from './health';

describe('isHealthy', () => {
	it('returns true when status is ok', () => {
		expect(isHealthy({ status: 'ok', version: '0.1.0' })).toBe(true);
	});

	it('returns false when status is not ok', () => {
		expect(isHealthy({ status: 'error', version: '0.1.0' })).toBe(false);
	});
});
