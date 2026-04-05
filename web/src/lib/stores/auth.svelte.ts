import type { Session, Subscription, User } from '@supabase/supabase-js';
import { supabase } from '$lib/supabase';

let session = $state<Session | null>(null);
let user = $state<User | null>(null);
let loading = $state(true);

let subscription: Subscription | null = null;
let initialized = false;

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

export async function initAuth() {
	if (initialized) return;
	initialized = true;

	try {
		const {
			data: { session: initialSession }
		} = await supabase.auth.getSession();
		session = initialSession;
		user = initialSession?.user ?? null;
		loading = false;

		const {
			data: { subscription: sub }
		} = supabase.auth.onAuthStateChange((_event, newSession) => {
			if (!initialized) return;
			session = newSession;
			user = newSession?.user ?? null;
		});

		subscription = sub;
	} catch {
		loading = false;
		initialized = false;
	}
}

export function cleanupAuth() {
	if (subscription) {
		subscription.unsubscribe();
		subscription = null;
	}
	session = null;
	user = null;
	loading = true;
	initialized = false;
}

export async function signInWithEmail(email: string, password: string) {
	const { error } = await supabase.auth.signInWithPassword({ email, password });
	if (error) throw error;
}

export async function signUpWithEmail(email: string, password: string) {
	const { error } = await supabase.auth.signUp({ email, password });
	if (error) throw error;
}

export async function signOut() {
	const { error } = await supabase.auth.signOut();
	if (error) throw error;
}
