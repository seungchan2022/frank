/// MVP5 M3: 즐겨찾기 아이템 타입.
/// 서버 Favorite 모델과 1:1 대응.
export interface Favorite {
	id: string;
	userId: string;
	title: string;
	url: string;
	snippet: string | null;
	source: string;
	publishedAt: string | null;
	tagId: string | null;
	summary: string | null;
	insight: string | null;
	likedAt: string | null;
	createdAt: string;
}

/// POST /me/favorites 요청 바디.
/// id, userId, createdAt은 서버가 채우므로 제외.
export interface AddFavoriteBody {
	title: string;
	url: string;
	snippet: string | null;
	source: string;
	published_at: string | null;
	tag_id: string | null;
	summary: string | null;
	insight: string | null;
}
