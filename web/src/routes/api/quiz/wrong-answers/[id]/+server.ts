// DELETE /api/quiz/wrong-answers/[id] — 오답 1건 삭제 프록시.
// Rust API DELETE /api/me/quiz/wrong-answers/{id} 를 중계한다.
// 인증 토큰은 safeGetSession()으로 서버 사이드에서 획득.

import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const API_BASE = (process.env.VITE_RUST_API_URL ?? 'http://localhost:8080').replace(/\/$/, '');

export const DELETE: RequestHandler = async ({ params, locals: { safeGetSession } }) => {
	const { session } = await safeGetSession();
	if (!session) {
		return json({ error: 'Unauthorized' }, { status: 401 });
	}

	const { id } = params;

	try {
		const upstream = await fetch(
			`${API_BASE}/api/me/quiz/wrong-answers/${encodeURIComponent(id)}`,
			{
				method: 'DELETE',
				headers: {
					Authorization: `Bearer ${session.access_token}`
				}
			}
		);

		if (!upstream.ok) {
			const data = await upstream.json().catch(() => null);
			return json(
				{
					error:
						(data as { error?: string } | null)?.error ??
						`Upstream error (${upstream.status})`
				},
				{ status: upstream.status }
			);
		}

		return new Response(null, { status: 204 });
	} catch (e) {
		const message = e instanceof Error ? e.message : 'Internal error';
		return json({ error: message }, { status: 500 });
	}
};
