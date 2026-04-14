import Foundation
import Observation

/// MVP7 M2: LikesFeature — 좋아요 상태 관리 + 키워드 추출 오케스트레이션.
///
/// FrankApp.swift에서 루트 소유로 생성 → FeedView, ArticleDetailView 공통 주입.
/// FavoritesFeature의 루트 공유 패턴과 동일하게 적용.
///
/// - likedUrls: 세션 동안 좋아요한 기사 URL Set
/// - like(feedItem:): API 호출 → likedUrls 추가 + lastKeywords 업데이트
/// - isLiked(_ url:): O(1) 조회
@Observable
@MainActor
final class LikesFeature {

    // MARK: - State

    /// 좋아요한 기사 URL Set. isLiked 조회 O(1).
    private(set) var likedUrls: Set<String> = []

    /// 가장 최근 like 처리에서 추출된 키워드.
    private(set) var lastKeywords: [String] = []

    /// like 실패 시 에러 메시지.
    private(set) var error: String? = nil

    // MARK: - Dependencies

    private let likes: any LikesPort

    // MARK: - Init

    init(likes: any LikesPort) {
        self.likes = likes
    }

    // MARK: - Actions

    /// 기사 좋아요 처리.
    /// - 이미 liked url이면 즉시 반환 (중복 방지).
    /// - API 성공 시 likedUrls 추가 + lastKeywords 업데이트.
    /// - API 실패 시 error 설정 (likedUrls 변경 없음).
    func like(feedItem: FeedItem) async {
        let urlString = feedItem.url.absoluteString

        // 중복 방지
        guard !likedUrls.contains(urlString) else { return }

        do {
            let result = try await likes.likeArticle(
                title: feedItem.title,
                snippet: feedItem.snippet,
                tagId: feedItem.tagId
            )
            likedUrls.insert(urlString)
            lastKeywords = result.keywords
        } catch {
            self.error = error.localizedDescription
        }
    }

    /// 좋아요 여부 확인.
    func isLiked(_ url: String) -> Bool {
        likedUrls.contains(url)
    }

    /// 에러 초기화.
    func clearError() {
        error = nil
    }
}
