import Foundation
@testable import Frank

final class MockRelatedPort: RelatedPort, @unchecked Sendable {
    var fetchCallCount = 0
    var shouldFail = false
    var stubbedItems: [FeedItem] = []

    func fetchRelated(title: String, snippet: String?) async throws -> [FeedItem] {
        fetchCallCount += 1
        if shouldFail { throw MockRelatedError.generic }
        return stubbedItems
    }
}

enum MockRelatedError: Error {
    case generic
}
