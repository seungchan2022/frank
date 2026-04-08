import Foundation
import Observation

enum ArticleDetailAction {
    case loadArticle
    case retrySummary
}

@Observable
@MainActor
final class ArticleDetailFeature {

    // MARK: - Data

    private(set) var article: Article? = nil

    // MARK: - Loading

    private(set) var isLoading = false

    // MARK: - Error

    private(set) var errorMessage: String? = nil

    // MARK: - Summary Timeout
    // 전체 요약 기준 (단일 요약 전환 시 10~15s로 줄일 것)
    private let summaryTimeoutSeconds: Double
    private(set) var summaryTimedOut: Bool = false
    private var summaryTimerTask: Task<Void, Never>?

    // MARK: - Dependencies

    private let articleId: UUID
    private let articlePort: any ArticlePort

    // MARK: - Init

    init(articleId: UUID, articlePort: any ArticlePort, summaryTimeoutSeconds: Double = 30) {
        self.articleId = articleId
        self.articlePort = articlePort
        self.summaryTimeoutSeconds = summaryTimeoutSeconds
    }

    // MARK: - Send

    func send(_ action: ArticleDetailAction) async {
        switch action {
        case .loadArticle:
            await loadArticle()
        case .retrySummary:
            summaryTimerTask?.cancel()
            summaryTimerTask = nil
            summaryTimedOut = false
            await loadArticle()
        }
    }

    // MARK: - State Transition Helpers

    private func beginLoading() {
        isLoading = true
        errorMessage = nil
        summaryTimedOut = false
        summaryTimerTask?.cancel()
        summaryTimerTask = nil
    }

    private func failLoading(_ message: String) {
        isLoading = false
        errorMessage = message
    }

    // MARK: - Core Logic

    private func loadArticle() async {
        beginLoading()
        do {
            article = try await articlePort.fetchArticle(id: articleId)
            isLoading = false
            if article?.summary == nil {
                startSummaryTimer()
            }
        } catch {
            failLoading("기사를 불러오지 못했습니다.")
        }
    }

    private func startSummaryTimer() {
        summaryTimerTask = Task { [weak self] in
            guard let self else { return }
            do {
                try await Task.sleep(for: .seconds(summaryTimeoutSeconds))
                summaryTimedOut = true
            } catch {
                // Task 취소됨 — no-op
            }
        }
    }

}
