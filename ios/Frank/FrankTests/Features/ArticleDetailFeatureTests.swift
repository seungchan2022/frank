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
        cache: SummarySessionCache? = nil
    ) -> (ArticleDetailFeature, MockSummarizePort) {
        let item = makeFeedItem(url: url)
        let resolvedCache = cache ?? SummarySessionCache()
        let feature = ArticleDetailFeature(feedItem: item, summarize: port, cache: resolvedCache)
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

    @Test("feedItem이 올바르게 보유됨")
    func feedItemIsRetained() {
        let item = makeFeedItem(url: "https://example.com/test")
        let port = MockSummarizePort()
        let feature = ArticleDetailFeature(feedItem: item, summarize: port)

        #expect(feature.feedItem.title == "Test Article")
        #expect(feature.feedItem.url.absoluteString == "https://example.com/test")
    }
}
