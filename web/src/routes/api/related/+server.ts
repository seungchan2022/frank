// GET /api/related — 연관 기사 프록시.
// Rust API GET /api/me/articles/related?title=...&snippet=... 를 중계한다.
// 인증 토큰은 safeGetSession()으로 서버 사이드에서 획득.

import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const API_BASE = (process.env.VITE_RUST_API_URL ?? 'http://localhost:8080').replace(/\/$/, '');

export const GET: RequestHandler = async ({ url, locals: { safeGetSession } }) => {
	const { session } = await safeGetSession();
	if (!session) {
		return json({ error: 'Unauthorized' }, { status: 401 });
	}

	const title = url.searchParams.get('title') ?? '';
	const snippet = url.searchParams.get('snippet') ?? '';

	const upstream_url = new URL(`${API_BASE}/api/me/articles/related`);
	if (title) upstream_url.searchParams.set('title', title);
	if (snippet) upstream_url.searchParams.set('snippet', snippet);

	try {
		const upstream = await fetch(upstream_url.toString(), {
			headers: {
				Authorization: `Bearer ${session.access_token}`
			}
		});

		const data = await upstream.json().catch(() => null);

		if (!upstream.ok) {
			return json(
				{
					error:
						(data as { error?: string } | null)?.error ?? `Upstream error (${upstream.status})`
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
