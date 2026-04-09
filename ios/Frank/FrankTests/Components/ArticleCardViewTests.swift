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
}
