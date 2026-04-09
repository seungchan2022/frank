// Re-export shared client-side types used by the ApiClient interface.
// 진실의 원천: progress/260407_API_SPEC.md
export type { Article, FeedItem } from '$lib/types/article';
export type { Tag } from '$lib/types/tag';

export interface Profile {
	id: string;
	display_name: string | null;
	onboarding_completed: boolean;
}

export interface ProfilePatch {
	display_name?: string;
	onboarding_completed?: boolean;
}

export interface FetchArticlesOptions {
	offset?: number;
	limit?: number;
	tagId?: string;
}
