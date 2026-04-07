// Server-side logout form action.
// supabase.auth.signOut()이 server cookies.setAll로 만료 쿠키를 보내 cookie를 정리한다.

import { redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async () => {
	throw redirect(303, '/login');
};

export const actions: Actions = {
	default: async ({ locals: { supabase } }) => {
		await supabase.auth.signOut();
		throw redirect(303, '/login');
	}
};
