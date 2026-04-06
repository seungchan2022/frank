import Foundation
@testable import Frank

final class MockCollectPort: CollectPort, @unchecked Sendable {
    var collectError: Error?
    var summarizeError: Error?
    var collectResult: Int = 0
    var summarizeResult: Int = 0

    var triggerCollectCallCount = 0
    var triggerSummarizeCallCount = 0

    func triggerCollect() async throws -> Int {
        triggerCollectCallCount += 1
        if let error = collectError { throw error }
        return collectResult
    }

    func triggerSummarize() async throws -> Int {
        triggerSummarizeCallCount += 1
        if let error = summarizeError { throw error }
        return summarizeResult
    }
}
