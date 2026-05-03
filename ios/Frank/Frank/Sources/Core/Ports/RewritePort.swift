import Foundation

/// POST /api/me/rewrite — url + title → RewriteResult.
/// occupation 미설정 시 서버가 400(occupation_required) 에러 반환.
protocol RewritePort: Sendable {
    func rewrite(url: String, title: String) async throws -> RewriteResult
}
