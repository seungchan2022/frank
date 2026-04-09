import Foundation
import Observation

/// M2 디테일 뷰 phase.
enum DetailPhase: Equatable {
    case idle
    case loading
    case done(SummaryResult)
    case failed(String)

    var summaryResult: SummaryResult? {
        guard case .done(let result) = self else { return nil }
        return result
    }

    var errorMessage: String? {
        guard case .failed(let msg) = self else { return nil }
        return msg
    }
}

/// MVP5 M2: ArticleDetailFeature — FeedItem 보유 + 온디맨드 요약.
/// - `phase`: idle → loading → done | failed
/// - `loadSummary()`: 캐시 히트 시 API 호출 없이 즉시 반환
@Observable
@MainActor
final class ArticleDetailFeature {

    // MARK: - Data

    let feedItem: FeedItem
    private(set) var phase: DetailPhase = .idle

    // MARK: - Dependencies

    private let summarize: any SummarizePort
    private let cache: SummarySessionCache

    // MARK: - Init

    init(
        feedItem: FeedItem,
        summarize: any SummarizePort,
        cache: SummarySessionCache = .shared
    ) {
        self.feedItem = feedItem
        self.summarize = summarize
        self.cache = cache
    }

    // MARK: - Actions

    func loadSummary() async {
        let url = feedItem.url.absoluteString

        // 캐시 히트 — API 호출 없이 즉시 반환
        if let cached = cache.get(url) {
            phase = .done(cached)
            return
        }

        phase = .loading

        do {
            let result = try await summarize.summarize(url: url, title: feedItem.title)
            cache.set(url, result)
            phase = .done(result)
        } catch {
            phase = .failed(errorMessage(from: error))
        }
    }

    // MARK: - Private

    private func errorMessage(from error: Error) -> String {
        if let apiError = error as? APISummarizeError, apiError == .timeout {
            return "요약 요청이 시간을 초과했습니다. 다시 시도해주세요."
        }
        return "요약을 불러오지 못했습니다. 다시 시도해주세요."
    }
}
