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

    // MARK: - FeedItem fixtures (MVP5 M1: ephemeral, id 없음)

    static var feedItems: [FeedItem] {
        [feedItem1, feedItem2, feedItem3, feedItem4, feedItem5, feedItem6]
    }

    /// I-04 pull-to-refresh 2단계 fixture.
    /// noCache=true(pull-to-refresh) 호출 시 이 목록을 반환하여
    /// 새로고침 후 목록이 변경됨을 시뮬레이션한다.
    static var refreshedFeedItems: [FeedItem] {
        [feedItemRefresh1, feedItemRefresh2, feedItem1, feedItem2]
    }

    private static let feedItemRefresh1 = FeedItem(
        title: "[새로고침] OpenAI GPT-5 Technical Report",
        url: URL(string: "https://example.com/news/gpt5-technical-report")!,
        source: "tavily",
        publishedAt: parseDate("2026-04-06T09:00:00Z"),
        tagId: tagAIML,
        snippet: "OpenAI releases the technical report for GPT-5 with multimodal capabilities."
    )

    private static let feedItemRefresh2 = FeedItem(
        title: "[새로고침] Xcode 17 Beta 2 Release Notes",
        url: URL(string: "https://example.com/news/xcode-17-beta2")!,
        source: "exa",
        publishedAt: parseDate("2026-04-06T08:00:00Z"),
        tagId: tagIOS,
        snippet: "Xcode 17 Beta 2 brings performance improvements and new debugging tools."
    )

    private static let feedItem1 = FeedItem(
        title: "Anthropic Releases Claude 4.6 with 1M Context Window",
        url: URL(string: "https://example.com/news/claude-4-6-release")!,
        source: "tavily",
        publishedAt: parseDate("2026-04-05T14:00:00Z"),
        tagId: tagAIML,
        snippet: "Anthropic announces the latest Claude model with 1M token context window."
    )

    private static let feedItem2 = FeedItem(
        title: "SvelteKit 2.0 Stable Release Notes",
        url: URL(string: "https://example.com/news/sveltekit-2-stable")!,
        source: "exa",
        publishedAt: parseDate("2026-04-04T09:00:00Z"),
        tagId: tagWeb,
        snippet: "SvelteKit reaches stable 2.0 with improved routing and runes API."
    )

    private static let feedItem3 = FeedItem(
        title: "Swift 6.1 Concurrency Improvements",
        url: URL(string: "https://example.com/news/swift-6-1-concurrency")!,
        source: "firecrawl",
        publishedAt: parseDate("2026-04-03T15:30:00Z"),
        tagId: tagIOS,
        snippet: "Swift 6.1 brings strict concurrency checking."
    )

    private static let feedItem4 = FeedItem(
        title: "RAG vs Fine-tuning: When to Use Which",
        url: URL(string: "https://example.com/news/rag-vs-finetuning")!,
        source: "tavily",
        publishedAt: parseDate("2026-04-02T11:00:00Z"),
        tagId: tagAIML,
        snippet: "A practical guide on choosing between RAG and fine-tuning."
    )

    private static let feedItem5 = FeedItem(
        title: "YC W26 Batch Application Deadline",
        url: URL(string: "https://example.com/news/yc-w26-deadline")!,
        source: "exa",
        publishedAt: parseDate("2026-04-01T08:00:00Z"),
        tagId: tagStartup,
        snippet: "Y Combinator opens applications for Winter 2026 batch."
    )

    private static let feedItem6 = FeedItem(
        title: "Untagged Article Example",
        url: URL(string: "https://example.com/news/untagged")!,
        source: "tavily",
        publishedAt: parseDate("2026-03-30T00:00:00Z"),
        tagId: nil,
        snippet: "An article without a tag for testing edge cases."
    )

    // MARK: - QuizQuestion fixtures (MVP7 M4)

    static let quizQuestions: [QuizQuestion] = [
        QuizQuestion(
            question: "Swift의 주요 특징은?",
            options: ["타입 안전성", "동적 타입", "가비지 컬렉션", "싱글 스레드"],
            answerIndex: 0,
            explanation: "Swift는 타입 안전성을 강조하는 언어입니다."
        ),
        QuizQuestion(
            question: "SwiftUI에서 상태를 관리하는 property wrapper는?",
            options: ["@Binding", "@State", "@ObservedObject", "@Published"],
            answerIndex: 1,
            explanation: "@State는 뷰 로컬 상태를 관리합니다."
        ),
        QuizQuestion(
            question: "Combine 프레임워크의 Publisher가 아닌 것은?",
            options: ["Just", "Future", "Subject", "Observer"],
            answerIndex: 3,
            explanation: "Observer는 Combine의 Publisher가 아닙니다."
        )
    ]

    // MARK: - Helpers

    private static func parseDate(_ iso: String) -> Date {
        ISO8601DateFormatter().date(from: iso) ?? Date(timeIntervalSince1970: 0)
    }
}
// swiftlint:enable force_unwrapping
