import Foundation

/// In-memory CollectPort 구현.
///
/// 항상 1건 반환 (수집 성공 시뮬레이션).
struct MockCollectAdapter: CollectPort {

    init() {}

    func triggerCollect() async throws -> Int {
        1
    }
}
