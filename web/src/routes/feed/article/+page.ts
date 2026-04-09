import type { PageLoad } from './$types';
import type { FeedItem } from '$lib/types/article';

export interface ArticlePageState {
	feedItem: FeedItem;
}

export const load: PageLoad = (event) => {
	const { url } = event;
	// pushState로 전달된 feedItem (이미 메모리에 있으면 재호출 불필요)
	// SvelteKit 2 shallow routing: state는 런타임에 제공되나 생성 타입에 미포함 → 캐스팅
	const state = (event as unknown as { state: Partial<ArticlePageState> }).state ?? {};
	const feedItem = state.feedItem;

	// URL 파라미터에서 기본 정보 복원 (새로고침 대응)
	const rawUrl = url.searchParams.get('url') ?? '';
	const title = url.searchParams.get('title') ?? '';
	const source = url.searchParams.get('source') ?? '';

	return {
		feedItem: feedItem ?? {
			title,
			url: rawUrl,
			source,
			snippet: null,
			published_at: null,
			tag_id: null
		}
	};
};
