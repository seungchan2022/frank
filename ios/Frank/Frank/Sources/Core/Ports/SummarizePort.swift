import Foundation

/// URL 크롤링 + LLM 요약 요청 포트.
/// POST /api/me/summarize — url + title → SummaryResult
protocol SummarizePort: Sendable {
    func summarize(url: String, title: String) async throws -> SummaryResult
}
