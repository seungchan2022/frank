import Foundation

/// POST /api/me/rewrite 응답 모델.
/// rewrite: 직업 시각으로 재작성된 기사 요약.
struct RewriteResult: Equatable, Codable, Sendable {
    let rewrite: String
}
