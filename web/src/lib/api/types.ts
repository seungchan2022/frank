// Re-export shared client-side types used by the ApiClient interface.
// 진실의 원천: progress/260407_API_SPEC.md
export type { Article, FeedItem } from '$lib/types/article';
export type { Tag } from '$lib/types/tag';

export interface Profile {
	id: string;
	display_name: string | null;
	onboarding_completed: boolean;
	/// MVP15 M3: 직업 한 줄 (최대 50자). null = 미설정.
	occupation: string | null;
}

export interface ProfilePatch {
	display_name?: string;
	onboarding_completed?: boolean;
	/// MVP15 M3: 직업 한 줄 저장. 공백 문자열은 null로 처리 (서버에서 trim).
	occupation?: string | null;
}

export interface FetchArticlesOptions {
	offset?: number;
	limit?: number;
	tagId?: string;
}
