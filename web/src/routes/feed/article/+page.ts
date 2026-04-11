import type { PageLoad } from './$types';
import type { FeedItem } from '$lib/types/article';

export interface ArticlePageState {
	feedItem: FeedItem;
}

export const load: PageLoad = (event) => {
	const { url } = event;

	// URL 파라미터에서 전체 정보 복원 (새로고침 대응)
	// state.feedItem은 +page.svelte에서 $page.state로 직접 읽음 (state 우선)
	const rawUrl = url.searchParams.get('url') ?? '';
	const title = url.searchParams.get('title') ?? '';
	const source = url.searchParams.get('source') ?? '';
	const snippet = url.searchParams.get('snippet');
	const published_at = url.searchParams.get('published_at');
	const tag_id = url.searchParams.get('tag_id');

	return {
		fallbackItem: {
			title,
			url: rawUrl,
			source,
			snippet,
			published_at,
			tag_id
		} satisfies FeedItem
	};
};
