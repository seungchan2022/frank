import Foundation
@testable import Frank

final class MockLikesPort: LikesPort, @unchecked Sendable {
    var likeCallCount = 0
    var shouldFail = false
    var stubbedKeywords: [String] = ["iOS", "Swift", "SwiftUI"]
    var stubbedTotalLikes: Int = 1

    func likeArticle(title: String, snippet: String?) async throws -> LikeResult {
        likeCallCount += 1
        if shouldFail { throw MockLikesError.generic }
        return LikeResult(keywords: stubbedKeywords, totalLikes: stubbedTotalLikes)
    }
}

enum MockLikesError: Error {
    case generic
}
