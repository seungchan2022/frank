import Foundation

/// In-memory SummarizePort 구현 — FRANK_USE_MOCK=1 모드 전용.
/// 지연 후 fixture 요약 결과를 반환.
struct MockSummarizeAdapter: SummarizePort {
    func summarize(url: String, title: String) async throws -> SummaryResult {
        // 600ms 지연으로 실제 API 호출을 시뮬레이션
        try await Task.sleep(for: .milliseconds(600))
        return SummaryResult(
            summary: "Mock 요약: \(title)에 대한 AI 요약문입니다.",
            insight: "Mock 인사이트: 이 기사는 핵심 내용을 담고 있습니다."
        )
    }
}
