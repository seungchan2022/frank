import Testing
import Foundation
@testable import Frank

@Suite("ArticleDetailFeature Tests")
@MainActor
struct ArticleDetailFeatureTests {

    // MARK: - Test Helpers

    private func makeSUT(
        articleId: UUID = UUID(),
        articleDetail: Article? = nil,
        articles: [Article] = [],
        fetchError: Error? = nil
    ) -> (ArticleDetailFeature, MockArticlePort) {
        let port = MockArticlePort()
        port.articleDetail = articleDetail
        port.articles = articles
        port.fetchError = fetchError
        let feature = ArticleDetailFeature(articleId: articleId, articlePort: port)
        return (feature, port)
    }

    private func makeArticle(
        id: UUID = UUID(),
        title: String = "Test Article",
        summary: String? = "Test summary",
        insight: String? = "Test insight"
    ) -> Article {
        Article(
            id: id,
            title: title,
            url: URL(string: "https://example.com")!,
            source: "TestSource",
            publishedAt: Date(),
            summary: summary,
            insight: insight
        )
    }

    // MARK: - 1. 초기 상태 검증

    @Test("초기 상태: article=nil, isLoading=false, errorMessage=nil")
    func initialState() {
        let (sut, _) = makeSUT()

        #expect(sut.article == nil)
        #expect(sut.isLoading == false)
        #expect(sut.errorMessage == nil)
    }

    // MARK: - 2. loadArticle 성공

    @Test("loadArticle 성공: article 설정됨, isLoading=false")
    func loadArticleSuccess() async {
        let articleId = UUID()
        let article = makeArticle(id: articleId, title: "Loaded Article")

        let (sut, _) = makeSUT(
            articleId: articleId,
            articleDetail: article
        )

        await sut.send(.loadArticle)

        #expect(sut.article != nil)
        #expect(sut.article?.id == articleId)
        #expect(sut.article?.title == "Loaded Article")
        #expect(sut.isLoading == false)
        #expect(sut.errorMessage == nil)
    }

    // MARK: - 3. loadArticle 실패

    @Test("loadArticle 실패: errorMessage 설정됨, isLoading=false, article=nil")
    func loadArticleFailure() async {
        let (sut, _) = makeSUT(
            fetchError: URLError(.notConnectedToInternet)
        )

        await sut.send(.loadArticle)

        #expect(sut.errorMessage == "기사를 불러오지 못했습니다.")
        #expect(sut.isLoading == false)
        #expect(sut.article == nil)
    }

    // MARK: - 4. 재시도 시 에러 초기화

    @Test("재시도 시 에러 초기화: 실패 후 재시도하면 errorMessage가 nil로 리셋")
    func retryResetsError() async {
        let articleId = UUID()
        let article = makeArticle(id: articleId)
        let port = MockArticlePort()
        port.fetchError = URLError(.timedOut)
        let sut = ArticleDetailFeature(articleId: articleId, articlePort: port)

        // 첫 시도: 실패
        await sut.send(.loadArticle)
        #expect(sut.errorMessage != nil)

        // 재시도: 성공
        port.fetchError = nil
        port.articleDetail = article
        await sut.send(.loadArticle)

        #expect(sut.errorMessage == nil)
        #expect(sut.article != nil)
        #expect(sut.isLoading == false)
    }

    // MARK: - 5. fetchArticleCallCount 검증

    @Test("fetchArticleCallCount 검증: loadArticle 호출 시 port.fetchArticle 1회 호출")
    func fetchArticleCallCount() async {
        let articleId = UUID()
        let article = makeArticle(id: articleId)

        let (sut, port) = makeSUT(
            articleId: articleId,
            articleDetail: article
        )

        await sut.send(.loadArticle)

        #expect(port.fetchArticleCallCount == 1)
    }

    // MARK: - 6. 올바른 articleId로 호출 검증

    @Test("올바른 articleId로 호출 검증")
    func fetchArticleCalledWithCorrectId() async {
        let articleId = UUID()
        let article = makeArticle(id: articleId)
        let port = MockArticlePort()
        port.articleDetail = article
        let sut = ArticleDetailFeature(articleId: articleId, articlePort: port)

        await sut.send(.loadArticle)

        #expect(sut.article?.id == articleId)
        #expect(port.fetchArticleCallCount == 1)
    }

    // MARK: - 7. summary nil인 기사도 정상 로드

    @Test("summary nil인 기사도 정상 로드")
    func loadArticleWithNilSummary() async {
        let articleId = UUID()
        let article = makeArticle(id: articleId, summary: nil, insight: nil)

        let (sut, _) = makeSUT(
            articleId: articleId,
            articleDetail: article
        )

        await sut.send(.loadArticle)

        #expect(sut.article != nil)
        #expect(sut.article?.id == articleId)
        #expect(sut.article?.summary == nil)
        #expect(sut.article?.insight == nil)
        #expect(sut.isLoading == false)
        #expect(sut.errorMessage == nil)
    }
}
