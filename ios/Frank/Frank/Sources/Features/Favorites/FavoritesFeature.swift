import Foundation
import Observation

// MARK: - Error Messages

private enum FavoritesErrorMessage {
    static let conflict = "이미 즐겨찾기에 추가된 기사입니다."
    static let addFailed = "즐겨찾기 추가에 실패했습니다. 다시 시도해주세요."
    static let removeFailed = "즐겨찾기 해제에 실패했습니다. 다시 시도해주세요."
    static let loadFailed = "즐겨찾기를 불러오지 못했습니다. 다시 시도해주세요."
}

/// 즐겨찾기 phase.
enum FavoritesPhase: Equatable {
    case idle
    case loading
    case done
    case failed(String)

    var errorMessage: String? {
        guard case .failed(let msg) = self else { return nil }
        return msg
    }
}

/// MVP5 M3: FavoritesFeature — 즐겨찾기 CRUD + 상태 관리.
///
/// step-5 필수 수정 I 반영: `hasLoaded: Bool` 플래그.
/// - 빈 즐겨찾기 사용자 재호출 방지
/// - `likedUrls`는 items에서 computed (Set<String>)
///
/// 상태 분리 원칙:
/// - `phase`: 초기 로드 전용 (idle → loading → done/failed)
/// - `operationError`: add/remove 변이 실패 시 인라인 표시용
@Observable
@MainActor
final class FavoritesFeature {

    // MARK: - State

    private(set) var phase: FavoritesPhase = .idle
    private(set) var items: [FavoriteItem] = []
    private(set) var hasLoaded: Bool = false

    /// add/remove 변이 실패 시 인라인 에러 메시지.
    /// phase와 독립적으로 관리 — 목록 화면을 유지한 채 에러 표시 가능.
    private(set) var operationError: String? = nil

    /// items에서 url 추출한 Set. isLiked 조회에 O(1).
    var likedUrls: Set<String> {
        Set(items.map(\.url))
    }

    // MARK: - Dependencies

    private let favorites: any FavoritesPort

    // MARK: - Init

    init(favorites: any FavoritesPort) {
        self.favorites = favorites
    }

    // MARK: - Actions

    /// GET /me/favorites → items + likedUrls 갱신.
    /// hasLoaded=true이면 no-op (빈 즐겨찾기도 보호).
    func loadFavorites() async {
        guard !hasLoaded else { return }

        phase = .loading
        do {
            let result = try await favorites.listFavorites()
            items = result
            hasLoaded = true
            phase = .done
        } catch {
            phase = .failed(FavoritesErrorMessage.loadFailed)
        }
    }

    /// 즐겨찾기 여부 확인.
    func isLiked(_ url: String) -> Bool {
        likedUrls.contains(url)
    }

    /// 변이 에러 초기화 (뷰에서 에러 dismiss 시 호출).
    func clearOperationError() {
        operationError = nil
    }

    /// POST /me/favorites → items에 prepend.
    /// step-5 필수 수정 K 반영: summary/insight 명시적 전달.
    /// 실패 시 phase 대신 operationError에 기록 — 목록 화면 유지.
    func addFavorite(feedItem: FeedItem, summary: String?, insight: String?) async {
        operationError = nil
        do {
            let added = try await favorites.addFavorite(item: feedItem, summary: summary, insight: insight)
            items = [added] + items
        } catch {
            operationError = errorMessage(from: error)
        }
    }

    /// DELETE /me/favorites → items에서 제거.
    /// 실패 시 phase 대신 operationError에 기록 — 목록 화면 유지.
    func removeFavorite(url: String) async {
        operationError = nil
        do {
            try await favorites.deleteFavorite(url: url)
            items = items.filter { $0.url != url }
        } catch {
            operationError = FavoritesErrorMessage.removeFailed
        }
    }

    // MARK: - Private

    private func errorMessage(from error: Error) -> String {
        if let favError = error as? APIFavoritesError, favError == .conflict {
            return FavoritesErrorMessage.conflict
        }
        return FavoritesErrorMessage.addFailed
    }
}
