// 클라이언트/서버 양쪽에서 동작하는 layout load.
// 단순히 +layout.server.ts가 채운 session/user를 페이지 데이터로 전달한다.
// supabase 클라이언트 인스턴스는 lib/supabase.ts의 글로벌(브라우저) /
// hooks.server.ts의 event.locals(서버) 두 개로 분리.

import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ data, depends }) => {
	depends('supabase:auth');
	return {
		session: data.session,
		user: data.user
	};
};
