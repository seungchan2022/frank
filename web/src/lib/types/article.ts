export interface Article {
	id: string;
	user_id: string;
	tag_id: string | null;
	title: string;
	url: string;
	snippet: string | null;
	source: string;
	published_at: string | null;
	created_at: string | null;
}
