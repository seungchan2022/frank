import Foundation
import Observation

/// MVP5 M1: ArticleDetailFeature — FeedItem을 직접 보유.
/// fetchArticle(id:) 제거 — ephemeral 피드에서는 아이템이 이미 메모리에 있음.
/// M2에서 url 기반 요약 엔드포인트 추가 예정.
@Observable
@MainActor
final class ArticleDetailFeature {

    // MARK: - Data

    let feedItem: FeedItem

    // MARK: - Init

    init(feedItem: FeedItem) {
        self.feedItem = feedItem
    }
}
