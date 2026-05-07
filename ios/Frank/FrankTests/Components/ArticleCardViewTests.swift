import Testing
import Foundation
@testable import Frank

@Suite("ArticleCardView 데이터 바인딩")
struct ArticleCardViewTests {

    // MARK: - Helpers

    private func makeArticle(
        title: String = "Test Article",
        source: String = "TestSource",
        publishedAt: Date = Date(),
        urlSuffix: String = "article"
    ) -> Article {
        Article(
            title: title,
            url: URL(string: "https://example.com/\(urlSuffix)")!,
            source: source,
            publishedAt: publishedAt,
            tagId: UUID()
        )
    }

    // MARK: - title 표시

    @Test("title이 표시됨")
    func titleDisplayed() {
        let article = makeArticle(title: "English Title")
        let view = ArticleCardView(article: article)

        // accessibilityLabel이 title과 동일
        #expect(view.article.title == "English Title")
    }

    // MARK: - Article model fields

    @Test("snippet이 nil인 Article 생성 가능")
    func articleWithNilSnippet() {
        let article = Article(
            title: "Test",
            url: URL(string: "https://example.com/nil-snippet")!,
            source: "Source",
            publishedAt: Date(),
            tagId: UUID(),
            snippet: nil
        )

        #expect(article.snippet == nil)
    }

    @Test("snippet이 있는 Article 생성 가능")
    func articleWithSnippet() {
        let article = Article(
            title: "Test",
            url: URL(string: "https://example.com/with-snippet")!,
            source: "Source",
            publishedAt: Date(),
            tagId: UUID(),
            snippet: "리드 문장"
        )

        #expect(article.snippet == "리드 문장")
    }

    // MARK: - Date relative display

    @Test("상대 시간 표시 — 방금 전")
    func relativeDateRecent() {
        let date = Date()
        let display = ArticleCardView.relativeTimeText(date)

        #expect(!display.isEmpty)
    }

    @Test("상대 시간 표시 — 과거")
    func relativeDatePast() {
        let date = Date().addingTimeInterval(-3600)
        let display = ArticleCardView.relativeTimeText(date)

        #expect(!display.isEmpty)
    }

    // MARK: - 기본값 검증

    @Test("옵셔널 필드 없이 Article 생성 — 기본값 nil")
    func articleDefaultValues() {
        let article = Article(
            title: "Test",
            url: URL(string: "https://example.com/defaults")!,
            source: "Source",
            publishedAt: Date(),
            tagId: UUID()
        )

        #expect(article.snippet == nil)
    }

    // MARK: - E1: 태그 칩 파라미터 (MVP16 M4)

    @Test("E1: tagName 파라미터 있으면 ArticleCardView 생성 가능")
    func tagNamePresent_viewCreatable() {
        let article = makeArticle()
        let view = ArticleCardView(article: article, tagName: "AI/ML")

        #expect(view.tagName == "AI/ML")
    }

    @Test("E1: tagName nil이면 ArticleCardView 생성 가능 (태그 없는 기사)")
    func tagNameNil_viewCreatable() {
        let article = makeArticle()
        let view = ArticleCardView(article: article, tagName: nil)

        #expect(view.tagName == nil)
    }

    @Test("E1: tagName 파라미터 기본값 nil — 기존 호출부 하위 호환")
    func tagNameDefaultNil_backwardCompat() {
        let article = makeArticle()
        let view = ArticleCardView(article: article)

        #expect(view.tagName == nil)
    }
}
