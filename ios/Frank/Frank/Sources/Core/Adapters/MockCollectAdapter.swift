import Foundation

/// In-memory CollectPort 구현.
///
/// `scenario`에 따라 동작을 전환한다:
/// - `nil` (기본): 항상 1건 반환
/// - `"summarize_timeout"`: `triggerSummarize()` 시 URLError(.timedOut) throw
struct MockCollectAdapter: CollectPort {
    let scenario: String?

    init(scenario: String? = nil) {
        self.scenario = scenario
    }

    func triggerCollect() async throws -> Int {
        1
    }

    func triggerSummarize() async throws -> Int {
        if scenario == "summarize_timeout" {
            // 60초 대기 — FeedFeature의 summarizeTimeout 타이머(테스트 시 짧게 주입)가 먼저 발화
            // 이렇게 해야 phase=.summarizing 유지 상태에서 isSummarizingTimeout=true 배너가 노출됨
            try? await Task.sleep(for: .seconds(60))
            return 0
        }
        return 1
    }
}
