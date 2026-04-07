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
        // category nullable이지만 iOS 현재 모델은 non-optional → "" 사용
        // (M3에서 모델 정정 예정)
        Tag(id: tagDesign, name: "디자인", category: "")
    ]

    // MARK: - Profile fixture

    static let profile = Profile(
        id: mockUserId,
        email: "mock@frank.dev",
        onboardingCompleted: true
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
        summary: "Anthropic은 100만 토큰 컨텍스트와 향상된 추론 능력을 갖춘 Claude 4.6을 공개했다.",
        tagId: tagAIML,
        titleKo: "앤스로픽, 100만 토큰 컨텍스트의 Claude 4.6 공개",
        insight: "장문 컨텍스트는 복잡한 코드베이스 분석과 멀티 문서 추론에 결정적이다.",
        snippet: "Anthropic announces the latest Claude model with 1M token context window.",
        summarizedAt: parseDate("2026-04-06T10:30:00Z")
    )

    private static let article2 = Article(
        id: articleId2,
        title: "SvelteKit 2.0 Stable Release Notes",
        url: URL(string: "https://example.com/news/sveltekit-2-stable")!,
        source: "exa",
        publishedAt: parseDate("2026-04-04T09:00:00Z"),
        summary: "SvelteKit 2.0이 안정 버전으로 출시됐다. 새로운 runes API와 개선된 라우팅을 제공한다.",
        tagId: tagWeb,
        titleKo: "SvelteKit 2.0 정식 출시 노트",
        insight: "Svelte 5 runes는 React/Vue와 다른 멘탈 모델을 요구한다.",
        snippet: "SvelteKit reaches stable 2.0 with improved routing and runes API.",
        summarizedAt: parseDate("2026-04-06T11:00:00Z")
    )

    private static let article3 = Article(
        id: articleId3,
        title: "Swift 6.1 Concurrency Improvements",
        url: URL(string: "https://example.com/news/swift-6-1-concurrency")!,
        source: "firecrawl",
        publishedAt: parseDate("2026-04-03T15:30:00Z"),
        summary: "Swift 6.1은 strict concurrency 체크와 세련된 Sendable 준수를 도입했다.",
        tagId: tagIOS,
        titleKo: "Swift 6.1 동시성 개선 사항",
        insight: "Swift Concurrency는 컴파일 타임 안전성을 위한 패러다임 전환이다.",
        snippet: "Swift 6.1 brings strict concurrency checking.",
        summarizedAt: parseDate("2026-04-06T12:00:00Z")
    )

    private static let article4 = Article(
        id: articleId4,
        title: "RAG vs Fine-tuning: When to Use Which",
        url: URL(string: "https://example.com/news/rag-vs-finetuning")!,
        source: "tavily",
        publishedAt: parseDate("2026-04-02T11:00:00Z"),
        summary: nil,
        tagId: tagAIML,
        titleKo: "RAG vs Fine-tuning: 언제 무엇을 써야 하나",
        insight: nil,
        snippet: "A practical guide on choosing between RAG and fine-tuning.",
        summarizedAt: nil
    )

    private static let article5 = Article(
        id: articleId5,
        title: "YC W26 Batch Application Deadline",
        url: URL(string: "https://example.com/news/yc-w26-deadline")!,
        source: "exa",
        publishedAt: parseDate("2026-04-01T08:00:00Z"),
        summary: "Y Combinator는 2026년 겨울 배치 지원을 받기 시작했다. 마감일은 2026년 5월 1일이다.",
        tagId: tagStartup,
        titleKo: "YC W26 배치 지원 마감 안내",
        insight: "YC 지원 시 가장 중요한 것은 팀의 실행력과 명확한 문제 정의다.",
        snippet: "Y Combinator opens applications for Winter 2026 batch.",
        summarizedAt: parseDate("2026-04-06T13:00:00Z")
    )

    private static let article6 = Article(
        id: articleId6,
        title: "Untagged Article Example",
        url: URL(string: "https://example.com/news/untagged")!,
        source: "tavily",
        publishedAt: parseDate("2026-03-30T00:00:00Z"),
        summary: nil,
        tagId: nil,
        titleKo: nil,
        insight: nil,
        snippet: "An article without a tag for testing edge cases.",
        summarizedAt: nil
    )

    // MARK: - Helpers

    private static func parseDate(_ iso: String) -> Date {
        ISO8601DateFormatter().date(from: iso) ?? Date(timeIntervalSince1970: 0)
    }
}
// swiftlint:enable force_unwrapping
