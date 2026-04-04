export interface HealthStatus {
	status: string;
	version: string;
}

export function isHealthy(health: HealthStatus): boolean {
	return health.status === 'ok';
}
