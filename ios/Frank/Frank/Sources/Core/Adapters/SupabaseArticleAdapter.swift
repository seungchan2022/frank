import Foundation
import Supabase

/// MVP5 M1: Supabase articles 테이블 직접 조회 어댑터.
/// 피드는 Rust 서버의 GET /me/feed를 통해 검색 API에서 직접 가져오므로 이 어댑터는 사용하지 않음.
/// APIArticleAdapter가 프로덕션 어댑터로 대체함.
struct SupabaseArticleAdapter: ArticlePort {
    private let client: SupabaseClient

    init(client: SupabaseClient) {
        self.client = client
    }

    func fetchFeed(tagId: UUID?) async throws -> [FeedItem] {
        // MVP5 M1: 피드는 Rust 서버 GET /me/feed를 통해 가져옴.
        // 이 어댑터는 미사용 — APIArticleAdapter 사용.
        return []
    }
}
