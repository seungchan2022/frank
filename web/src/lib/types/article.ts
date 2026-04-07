export interface Article {
	id: string;
	user_id: string;
	tag_id: string | null;
	title: string;
	title_ko: string | null;
	url: string;
	snippet: string | null;
	source: string;
	search_query: string | null;
	published_at: string | null;
	created_at: string | null;
	summary: string | null;
	insight: string | null;
	summarized_at: string | null;
}
