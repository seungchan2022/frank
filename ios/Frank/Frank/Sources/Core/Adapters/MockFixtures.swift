import Foundation

// 진실의 원천: progress/fixtures/*.json (M1.5)
// Swift 모델에 맞춰 JSON 내용을 한 번에 표현. JSON 변경 시 본 파일도 같이 갱신.
//
// swiftlint:disable force_unwrapping
enum MockFixtures {
    // MARK: - IDs

    static let mockUserId = UUID(uuidString: "00000000-0000-0000-0000-000000000001")!

    static let tagAIML = UUID(uuidString: "11111111-1111-1111-1111-111111111111")!
    static let tagWeb = UUID(uuidString: "22222222-2222-2222-2222-222222222222")!
    static let tagIOS = UUID(uuidString: "33333333-3333-3333-3333-333333333333")!
    static let tagStartup = UUID(uuidString: "44444444-4444-4444-4444-444444444444")!
    static let tagDesign = UUID(uuidString: "55555555-5555-5555-5555-555555555555")!

    private static let articleId1 = UUID(uuidString: "aaaaaaa1-0000-0000-0000-000000000001")!
    private static let articleId2 = UUID(uuidString: "aaaaaaa2-0000-0000-0000-000000000002")!
    private static let articleId3 = UUID(uuidString: "aaaaaaa3-0000-0000-0000-000000000003")!
    private static let articleId4 = UUID(uuidString: "aaaaaaa4-0000-0000-0000-000000000004")!
    private static let articleId5 = UUID(uuidString: "aaaaaaa5-0000-0000-0000-000000000005")!
    private static let articleId6 = UUID(uuidString: "aaaaaaa6-0000-0000-0000-000000000006")!

    // MARK: - Tag fixtures

    static let tags: [Tag] = [
        Tag(id: tagAIML, name: "AI/ML", category: "기술"),
        Tag(id: tagWeb, name: "웹 개발", category: "기술"),
        Tag(id: tagIOS, name: "iOS 개발", category: "기술"),
        Tag(id: tagStartup, name: "스타트업", category: "비즈니스"),
        // category는 nullable — 카테고리 미지정 케이스 표현
        Tag(id: tagDesign, name: "디자인", category: nil)
    ]

    // MARK: - Profile fixtures

    static let profile = Profile(
        id: mockUserId,
        displayName: "Mock User",
        onboardingCompleted: true
    )

    /// 온보딩 미완료 신규 사용자 fixture (TC-02)
    static let newUserProfile = Profile(
        id: mockUserId,
        displayName: "Mock User",
        onboardingCompleted: false
    )

    // MARK: - Article fixtures

    static var articles: [Article] {
        [article1, article2, article3, article4, article5, article6]
    }

    private static let article1 = Article(
        id: articleId1,
        title: "Anthropic Releases Claude 4.6 with 1M Context Window",
        url: URL(string: "https://example.com/news/claude-4-6-release")!,
        source: "tavily",
        publishedAt: parseDate("2026-04-05T14:00:00Z"),
        tagId: tagAIML,
        snippet: "Anthropic announces the latest Claude model with 1M token context window."
    )

    private static let article2 = Article(
        id: articleId2,
        title: "SvelteKit 2.0 Stable Release Notes",
        url: URL(string: "https://example.com/news/sveltekit-2-stable")!,
        source: "exa",
        publishedAt: parseDate("2026-04-04T09:00:00Z"),
        tagId: tagWeb,
        snippet: "SvelteKit reaches stable 2.0 with improved routing and runes API."
    )

    private static let article3 = Article(
        id: articleId3,
        title: "Swift 6.1 Concurrency Improvements",
        url: URL(string: "https://example.com/news/swift-6-1-concurrency")!,
        source: "firecrawl",
        publishedAt: parseDate("2026-04-03T15:30:00Z"),
        tagId: tagIOS,
        snippet: "Swift 6.1 brings strict concurrency checking."
    )

    private static let article4 = Article(
        id: articleId4,
        title: "RAG vs Fine-tuning: When to Use Which",
        url: URL(string: "https://example.com/news/rag-vs-finetuning")!,
        source: "tavily",
        publishedAt: parseDate("2026-04-02T11:00:00Z"),
        tagId: tagAIML,
        snippet: "A practical guide on choosing between RAG and fine-tuning."
    )

    private static let article5 = Article(
        id: articleId5,
        title: "YC W26 Batch Application Deadline",
        url: URL(string: "https://example.com/news/yc-w26-deadline")!,
        source: "exa",
        publishedAt: parseDate("2026-04-01T08:00:00Z"),
        tagId: tagStartup,
        snippet: "Y Combinator opens applications for Winter 2026 batch."
    )

    private static let article6 = Article(
        id: articleId6,
        title: "Untagged Article Example",
        url: URL(string: "https://example.com/news/untagged")!,
        source: "tavily",
        publishedAt: parseDate("2026-03-30T00:00:00Z"),
        tagId: nil,
        snippet: "An article without a tag for testing edge cases."
    )

    // MARK: - Helpers

    private static func parseDate(_ iso: String) -> Date {
        ISO8601DateFormatter().date(from: iso) ?? Date(timeIntervalSince1970: 0)
    }
}
// swiftlint:enable force_unwrapping
