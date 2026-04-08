import Foundation
import Observation

enum ArticleDetailAction {
    case loadArticle
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

    // MARK: - Dependencies

    private let articleId: UUID
    private let articlePort: any ArticlePort

    // MARK: - Init

    init(articleId: UUID, articlePort: any ArticlePort) {
        self.articleId = articleId
        self.articlePort = articlePort
    }

    // MARK: - Send

    func send(_ action: ArticleDetailAction) async {
        switch action {
        case .loadArticle:
            await loadArticle()
        }
    }

    // MARK: - State Transition Helpers

    private func beginLoading() {
        isLoading = true
        errorMessage = nil
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
        } catch {
            failLoading("기사를 불러오지 못했습니다.")
        }
    }
}
