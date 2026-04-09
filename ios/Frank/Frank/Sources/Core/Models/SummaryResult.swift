import Foundation

/// POST /me/summarize 응답 모델.
/// summary: LLM이 생성한 요약문, insight: 핵심 인사이트.
struct SummaryResult: Equatable, Codable, Sendable {
    let summary: String
    let insight: String
}
