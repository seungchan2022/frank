import Foundation
@testable import Frank

final class MockRewritePort: RewritePort, @unchecked Sendable {
    var rewriteResult: Result<RewriteResult, Error> = .success(
        RewriteResult(rewrite: "Mock 재작성 결과")
    )
    var rewriteCallCount = 0

    func rewrite(url: String, title: String) async throws -> RewriteResult {
        rewriteCallCount += 1
        return try rewriteResult.get()
    }
}
