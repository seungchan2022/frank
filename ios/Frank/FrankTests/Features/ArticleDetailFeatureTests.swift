import Testing
import Foundation
@testable import Frank

@Suite("ArticleDetailFeature Tests — M2")
@MainActor
struct ArticleDetailFeatureTests {

    // MARK: - Helpers

    private func makeFeedItem(url: String = "https://example.com/article") -> FeedItem {
        FeedItem(
            title: "Test Article",
            url: URL(string: url)!,
            source: "TestSource",
            publishedAt: Date(),
            tagId: nil,
            snippet: "Test snippet"
        )
    }

    private func makeSUT(
        url: String = "https://example.com/article",
        port: MockSummarizePort = MockSummarizePort(),
        rewritePort: MockRewritePort = MockRewritePort(),
        authPort: MockAuthPort = MockAuthPort(),
        cache: SummarySessionCache? = nil
    ) -> (ArticleDetailFeature, MockSummarizePort) {
        let item = makeFeedItem(url: url)
        let resolvedCache = cache ?? SummarySessionCache()
        let feature = ArticleDetailFeature(
            feedItem: item,
            summarize: port,
            rewrite: rewritePort,
            auth: authPort,
            cache: resolvedCache
        )
        return (feature, port)
    }

    // MARK: - 1. 초기 상태

    @Test("초기 상태: phase=idle")
    func initialState() {
        let (sut, _) = makeSUT()
        #expect(sut.phase == .idle)
    }

    // MARK: - 2. 캐시 미스 → API 호출 → done

    @Test("캐시 미스: API 1회 호출 → phase=done")
    func cacheMissCallsAPI() async {
        let port = MockSummarizePort()
        let (sut, _) = makeSUT(port: port)

        await sut.loadSummary()

        #expect(port.callCount == 1)
        if case .done(let result) = sut.phase {
            #expect(result.summary == "Mock summary text.")
            #expect(result.insight == "Mock insight text.")
        } else {
            Issue.record("Expected done phase, got \(sut.phase)")
        }
    }

    // MARK: - 3. 캐시 히트 → API 호출 없음

    @Test("캐시 히트: API 호출 0회 → phase=done")
    func cacheHitSkipsAPI() async {
        let port = MockSummarizePort()
        let cache = SummarySessionCache()
        let url = "https://example.com/cached"
        let cached = SummaryResult(summary: "Cached summary", insight: "Cached insight")
        cache.set(url, cached)

        let (sut, _) = makeSUT(url: url, port: port, cache: cache)
        await sut.loadSummary()

        #expect(port.callCount == 0)
        if case .done(let result) = sut.phase {
            #expect(result.summary == "Cached summary")
            #expect(result.insight == "Cached insight")
        } else {
            Issue.record("Expected done phase, got \(sut.phase)")
        }
    }

    // MARK: - 4. API 실패 → failed

    @Test("API 실패: phase=failed, canRetry 가능")
    func apiFailureSetsFailedPhase() async {
        let port = MockSummarizePort()
        port.error = URLError(.notConnectedToInternet)
        let (sut, _) = makeSUT(port: port)

        await sut.loadSummary()

        if case .failed(let msg) = sut.phase {
            #expect(!msg.isEmpty)
        } else {
            Issue.record("Expected failed phase, got \(sut.phase)")
        }
    }

    // MARK: - 5. 타임아웃 에러 → failed + 타임아웃 메시지

    @Test("타임아웃 에러: phase=failed, 타임아웃 메시지 포함")
    func timeoutErrorSetsTimeoutMessage() async {
        let port = MockSummarizePort()
        port.error = APISummarizeError.timeout
        let (sut, _) = makeSUT(port: port)

        await sut.loadSummary()

        if case .failed(let msg) = sut.phase {
            #expect(msg.contains("시간을 초과"))
        } else {
            Issue.record("Expected failed phase, got \(sut.phase)")
        }
    }

    // MARK: - 6. 재시도

    @Test("재시도: 실패 후 loadSummary 재호출 → done")
    func retryAfterFailure() async {
        let port = MockSummarizePort()
        port.error = URLError(.timedOut)
        let (sut, _) = makeSUT(port: port)

        await sut.loadSummary()
        #expect(sut.phase.errorMessage != nil)

        // 재시도
        port.error = nil
        await sut.loadSummary()

        if case .done = sut.phase {
            // 성공
        } else {
            Issue.record("Expected done phase after retry, got \(sut.phase)")
        }
    }

    // MARK: - 7. 성공 후 결과가 캐시에 저장됨

    @Test("성공 후 결과가 캐시에 저장: 두 번째 호출 시 API 호출 0")
    func successfulSummaryIsCached() async {
        let port = MockSummarizePort()
        port.result = SummaryResult(summary: "Cached summary", insight: "Cached insight")
        let cache = SummarySessionCache()
        let url = "https://example.com/article"
        let (sut, _) = makeSUT(url: url, port: port, cache: cache)

        await sut.loadSummary()
        #expect(port.callCount == 1)
        #expect(cache.get(url) != nil)

        // 두 번째 호출 — 캐시 히트
        await sut.loadSummary()
        // 캐시 히트이므로 callCount 는 여전히 1
        #expect(port.callCount == 1)
    }

    // MARK: - 8. feedItem 접근

    // MARK: - MVP15 M3: Rewrite

    @Test("MVP15 M3: loadRewrite 성공 → rewritePhase=done")
    func loadRewrite_success() async {
        let rewritePort = MockRewritePort()
        rewritePort.rewriteResult = .success(RewriteResult(rewrite: "재작성 결과"))
        let (sut, _) = makeSUT(rewritePort: rewritePort)

        await sut.loadRewrite()

        if case .done(let result) = sut.rewritePhase {
            #expect(result.rewrite == "재작성 결과")
        } else {
            Issue.record("Expected done rewritePhase, got \(sut.rewritePhase)")
        }
        #expect(rewritePort.rewriteCallCount == 1)
    }

    @Test("MVP15 M3: loadRewrite 실패 → rewritePhase=failed")
    func loadRewrite_failure() async {
        let rewritePort = MockRewritePort()
        rewritePort.rewriteResult = .failure(URLError(.networkConnectionLost))
        let (sut, _) = makeSUT(rewritePort: rewritePort)

        await sut.loadRewrite()

        if case .failed(let msg) = sut.rewritePhase {
            #expect(!msg.isEmpty)
        } else {
            Issue.record("Expected failed rewritePhase, got \(sut.rewritePhase)")
        }
    }

    @Test("MVP15 M3: loadRewrite occupation 미설정 → occupationRequired 에러 메시지")
    func loadRewrite_occupationRequired() async {
        let rewritePort = MockRewritePort()
        rewritePort.rewriteResult = .failure(APIRewriteError.occupationRequired)
        let (sut, _) = makeSUT(rewritePort: rewritePort)

        await sut.loadRewrite()

        if case .failed(let msg) = sut.rewritePhase {
            #expect(msg.contains("직업"))
        } else {
            Issue.record("Expected failed rewritePhase, got \(sut.rewritePhase)")
        }
    }

    @Test("MVP15 M3: loadRewrite 단일 호출 후 API 1회만 호출됨")
    func loadRewrite_singleCallInvokesAPIOnce() async {
        let rewritePort = MockRewritePort()
        let (sut, _) = makeSUT(rewritePort: rewritePort)

        await sut.loadRewrite()

        #expect(rewritePort.rewriteCallCount == 1)
    }

    @Test("MVP15 M3: loadRewrite 연속 호출 시 done 상태에서도 재호출 가능")
    func loadRewrite_canRetryAfterDone() async {
        let rewritePort = MockRewritePort()
        rewritePort.rewriteResult = .success(RewriteResult(rewrite: "첫 번째 재작성"))
        let (sut, _) = makeSUT(rewritePort: rewritePort)

        await sut.loadRewrite()
        #expect(rewritePort.rewriteCallCount == 1)

        // done 상태 이후 재호출 가능 — loading 가드는 loading 중에만 막음
        rewritePort.rewriteResult = .success(RewriteResult(rewrite: "두 번째 재작성"))
        await sut.loadRewrite()
        #expect(rewritePort.rewriteCallCount == 2)
    }

    @Test("MVP15 M3: loadRewrite loading 중 중복 호출 무시")
    func loadRewrite_loadingGuardBlocksDuplicateCall() async {
        // 느린 Mock — continuation이 resume될 때까지 대기
        final class SlowRewritePort: RewritePort, @unchecked Sendable {
            var callCount = 0
            // resume 트리거: 외부에서 호출해 첫 번째 await를 완료시킴
            var continuation: CheckedContinuation<RewriteResult, Error>?

            func rewrite(url: String, title: String) async throws -> RewriteResult {
                callCount += 1
                return try await withCheckedThrowingContinuation { cont in
                    self.continuation = cont
                }
            }
        }

        let slowPort = SlowRewritePort()
        let item = makeFeedItem()
        let feature = ArticleDetailFeature(
            feedItem: item,
            summarize: MockSummarizePort(),
            rewrite: slowPort,
            auth: MockAuthPort()
        )

        // 첫 번째 loadRewrite — continuation이 resume될 때까지 멈춤
        let firstTask = Task { await feature.loadRewrite() }

        // 첫 번째 Task가 loading 상태에 진입하도록 잠시 양보
        await Task.yield()
        await Task.yield()

        // loading 상태에서 두 번째 호출 — 가드에 걸려 callCount 증가 없음
        await feature.loadRewrite()

        // 첫 번째 Task를 완료시킴
        slowPort.continuation?.resume(returning: RewriteResult(rewrite: "결과"))
        await firstTask.value

        // API는 정확히 1회만 호출되어야 함
        #expect(slowPort.callCount == 1)
        if case .done(let result) = feature.rewritePhase {
            #expect(result.rewrite == "결과")
        } else {
            Issue.record("Expected done rewritePhase after resume, got \(feature.rewritePhase)")
        }
    }

    @Test("MVP15 M3: loadUserProfile → userProfile 세팅")
    func loadUserProfile_success() async {
        let authPort = MockAuthPort()
        authPort.currentProfileResult = Profile(
            id: UUID(),
            displayName: "test",
            onboardingCompleted: true,
            occupation: "iOS 개발자"
        )
        let (sut, _) = makeSUT(authPort: authPort)

        await sut.loadUserProfile()

        #expect(sut.userProfile?.occupation == "iOS 개발자")
        #expect(authPort.currentProfileCallCount == 1)
    }

    // MARK: - 8. feedItem 접근

    @Test("feedItem이 올바르게 보유됨")
    func feedItemIsRetained() {
        let item = makeFeedItem(url: "https://example.com/test")
        let port = MockSummarizePort()
        let feature = ArticleDetailFeature(
            feedItem: item,
            summarize: port,
            rewrite: MockRewritePort(),
            auth: MockAuthPort()
        )

        #expect(feature.feedItem.title == "Test Article")
        #expect(feature.feedItem.url.absoluteString == "https://example.com/test")
    }
}
