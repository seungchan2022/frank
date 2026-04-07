import Foundation

/// In-memory CollectPort 구현. 호출 시 항상 1건 반환.
struct MockCollectAdapter: CollectPort {
    func triggerCollect() async throws -> Int {
        1
    }

    func triggerSummarize() async throws -> Int {
        1
    }
}
