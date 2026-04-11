import Foundation

/// MockRelatedAdapter — 개발/테스트용 고정 연관 기사 반환.
/// FRANK_USE_MOCK=1 환경변수 또는 Preview 시 사용.
struct MockRelatedAdapter: RelatedPort {
    func fetchRelated(title: String, snippet: String?) async throws -> [FeedItem] {
        guard let url1 = URL(string: "https://example.com/related/1"),
              let url2 = URL(string: "https://example.com/related/2") else {
            return []
        }
        return [
            FeedItem(
                title: "연관 기사 1: Swift 최신 동향",
                url: url1,
                source: "TechNews",
                publishedAt: Date().addingTimeInterval(-3600),
                snippet: "Swift 5.10에서 추가된 새로운 기능들을 알아봅니다."
            ),
            FeedItem(
                title: "연관 기사 2: iOS 개발 트렌드",
                url: url2,
                source: "DevBlog",
                publishedAt: Date().addingTimeInterval(-7200),
                snippet: "2025년 iOS 개발 생태계 변화와 주요 트렌드 분석."
            ),
        ]
    }
}
