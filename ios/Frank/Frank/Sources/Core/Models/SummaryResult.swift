import Foundation

/// POST /me/summarize 응답 모델.
/// summary: LLM이 생성한 요약문.
/// insight: 직업 맞춤 인사이트 (occupation 미설정 시 nil).
struct SummaryResult: Equatable, Codable, Sendable {
    let summary: String
    let insight: String?
}
