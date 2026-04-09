import Foundation
@testable import Frank

final class MockCollectPort: CollectPort, @unchecked Sendable {
    var collectError: Error?
    var collectResult: Int = 0

    var triggerCollectCallCount = 0

    func triggerCollect() async throws -> Int {
        triggerCollectCallCount += 1
        if let error = collectError { throw error }
        return collectResult
    }
}
