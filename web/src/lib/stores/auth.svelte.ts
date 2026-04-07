// Server-driven auth store.
//
// M2 ST-4 (옵션 B): client-side supabase 호출 제거.
// session/user는 +layout.server.ts → +layout.svelte → setAuth(...)로 hydration.
// signIn/signOut은 server form actions(routes/login, routes/logout)으로 처리.
// 페이지 컴포넌트는 getAuth().isAuthenticated/user/session으로 접근한다.

import type { Session, User } from '@supabase/supabase-js';

let session = $state<Session | null>(null);
let user = $state<User | null>(null);
let loading = $state(false);

export function setAuth(data: { session: Session | null; user: User | null }): void {
	session = data.session;
	user = data.user;
	loading = false;
}

export function getAuth() {
	return {
		get session() {
			return session;
		},
		get user() {
			return user;
		},
		get loading() {
			return loading;
		},
		get isAuthenticated() {
			return !!session;
		}
	};
}
