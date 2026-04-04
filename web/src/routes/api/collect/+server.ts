import { json } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import type { RequestHandler } from './$types';

const API_SERVER_URL = env.API_SERVER_URL ?? 'http://localhost:8080';

export const POST: RequestHandler = async ({ request }) => {
	const authorization = request.headers.get('Authorization');
	if (!authorization) {
		return json({ error: 'Missing Authorization header' }, { status: 401 });
	}

	try {
		const response = await fetch(`${API_SERVER_URL}/api/me/collect`, {
			method: 'POST',
			headers: {
				Authorization: authorization,
				'Content-Type': 'application/json'
			}
		});

		const data = await response.json();

		if (!response.ok) {
			return json(
				{ error: data.error ?? 'Upstream server error' },
				{ status: response.status }
			);
		}

		return json(data);
	} catch (err) {
		const message = err instanceof Error ? err.message : 'Unknown error';
		return json({ error: `Failed to reach API server: ${message}` }, { status: 502 });
	}
};
