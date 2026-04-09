import Foundation
@testable import Frank

final class MockSummarizePort: SummarizePort, @unchecked Sendable {
    var result: SummaryResult = SummaryResult(
        summary: "Mock summary text.",
        insight: "Mock insight text."
    )
    var error: Error?
    var callCount = 0
    var lastURL: String?
    var lastTitle: String?

    func summarize(url: String, title: String) async throws -> SummaryResult {
        callCount += 1
        lastURL = url
        lastTitle = title
        if let error { throw error }
        return result
    }
}
