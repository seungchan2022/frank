import Testing
import Foundation
@testable import Frank

@Suite("ArticleCardView 데이터 바인딩")
struct ArticleCardViewTests {

    // MARK: - Helpers

    private func makeArticle(
        title: String = "Test Article",
        titleKo: String? = nil,
        source: String = "TestSource",
        publishedAt: Date = Date(),
        summary: String? = nil,
        insight: String? = nil
    ) -> Article {
        Article(
            id: UUID(),
            title: title,
            url: URL(string: "https://example.com")!,
            source: source,
            publishedAt: publishedAt,
            summary: summary,
            tagId: UUID(),
            titleKo: titleKo,
            insight: insight
        )
    }

    // MARK: - displayTitle fallback

    @Test("titleKo가 있으면 titleKo를 displayTitle로 사용")
    func displayTitleUsesKorean() {
        let article = makeArticle(title: "English Title", titleKo: "한국어 제목")
        let view = ArticleCardView(article: article)

        #expect(view.displayTitle == "한국어 제목")
    }

    @Test("titleKo가 nil이면 title을 displayTitle로 사용")
    func displayTitleFallsBackToTitle() {
        let article = makeArticle(title: "English Title", titleKo: nil)
        let view = ArticleCardView(article: article)

        #expect(view.displayTitle == "English Title")
    }

    // MARK: - Article model fields

    @Test("summary가 nil인 Article 생성 가능")
    func articleWithNilSummary() {
        let article = makeArticle(summary: nil)

        #expect(article.summary == nil)
    }

    @Test("summary가 있는 Article 생성 가능")
    func articleWithSummary() {
        let article = makeArticle(summary: "요약 텍스트")

        #expect(article.summary == "요약 텍스트")
    }

    @Test("insight가 nil인 Article 생성 가능")
    func articleWithNilInsight() {
        let article = makeArticle(insight: nil)

        #expect(article.insight == nil)
    }

    @Test("insight가 있는 Article 생성 가능")
    func articleWithInsight() {
        let article = makeArticle(insight: "인사이트 텍스트")

        #expect(article.insight == "인사이트 텍스트")
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

    // MARK: - Backward compatibility

    @Test("새 필드 없이 Article 생성 — 기본값 nil")
    func articleBackwardCompatibility() {
        let article = Article(
            id: UUID(),
            title: "Test",
            url: URL(string: "https://example.com")!,
            source: "Source",
            publishedAt: Date(),
            tagId: UUID()
        )

        #expect(article.titleKo == nil)
        #expect(article.insight == nil)
        #expect(article.summary == nil)
    }
}
