import Foundation
@testable import Frank

final class MockCollectPort: CollectPort, @unchecked Sendable {
    var collectError: Error?
    var summarizeError: Error?

    var triggerCollectCallCount = 0
    var triggerSummarizeCallCount = 0

    func triggerCollect() async throws {
        triggerCollectCallCount += 1
        if let error = collectError { throw error }
    }

    func triggerSummarize() async throws {
        triggerSummarizeCallCount += 1
        if let error = summarizeError { throw error }
    }
}
