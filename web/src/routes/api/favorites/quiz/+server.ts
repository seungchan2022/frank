// POST /api/favorites/quiz — 즐겨찾기 기사 퀴즈 생성 프록시.
// Rust API POST /api/me/favorites/quiz 를 중계한다.
// 인증 토큰은 safeGetSession()으로 서버 사이드에서 획득.

import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const API_BASE = (process.env.VITE_RUST_API_URL ?? 'http://localhost:8080').replace(/\/$/, '');

export const POST: RequestHandler = async ({ request, locals: { safeGetSession } }) => {
	const { session } = await safeGetSession();
	if (!session) {
		return json({ error: 'Unauthorized' }, { status: 401 });
	}

	let body: unknown;
	try {
		body = await request.json();
	} catch {
		return json({ error: 'Invalid JSON body' }, { status: 400 });
	}

	try {
		const upstream = await fetch(`${API_BASE}/api/me/favorites/quiz`, {
			method: 'POST',
			headers: {
				Authorization: `Bearer ${session.access_token}`,
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(body)
		});

		const data = await upstream.json().catch(() => null);

		if (!upstream.ok) {
			return json(
				{
					error:
						(data as { error?: string } | null)?.error ??
						`Upstream error (${upstream.status})`
				},
				{ status: upstream.status }
			);
		}

		return json(data);
	} catch (e) {
		const message = e instanceof Error ? e.message : 'Internal error';
		return json({ error: message }, { status: 500 });
	}
};
