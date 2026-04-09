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

/// MVP5 M1: 피드 아이템 — ephemeral, DB에 저장되지 않음.
/// id 없음 — 클라이언트는 url을 기사 식별자로 사용.
export interface FeedItem {
	title: string;
	url: string;
	snippet: string | null;
	source: string;
	published_at: string | null;
	tag_id: string | null;
}
