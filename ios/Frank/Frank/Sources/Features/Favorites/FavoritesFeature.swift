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
/// MVP11 M4: TagPort 주입 → tags + filteredItems(selectedTagId:) 추가.
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

    /// MVP11 M4: 즐겨찾기 기사에 실제 존재하는 태그만 표시.
    private(set) var tags: [Tag] = []

    /// ST1 BUG-D: loadFavorites 시 fetchAllTags 결과를 보관.
    /// addFavorite/removeFavorite 후 recomputeTags()에서 재사용.
    private var allTagsCache: [Tag] = []

    /// items에서 url 추출한 Set. isLiked 조회에 O(1).
    var likedUrls: Set<String> {
        Set(items.map(\.url))
    }

    /// MVP11 M4: selectedTagId 기준 필터된 즐겨찾기 목록.
    /// selectedTagId는 FavoritesView @State 단일 소스 — Feature는 파라미터로 받아 계산만.
    func filteredItems(selectedTagId: UUID?) -> [FavoriteItem] {
        guard let tagId = selectedTagId else { return items }
        return items.filter { $0.tagId == tagId }
    }

    // MARK: - Dependencies

    private let favorites: any FavoritesPort
    private let tagPort: any TagPort

    // MARK: - Init

    init(favorites: any FavoritesPort, tag: any TagPort) {
        self.favorites = favorites
        self.tagPort = tag
    }

    // MARK: - Actions

    /// GET /me/favorites → items + likedUrls + tags 갱신.
    /// hasLoaded=true이면 no-op (빈 즐겨찾기도 보호).
    ///
    /// MVP11 M4: items 로드 성공 후 fetchAllTags()로 태그 목록 갱신.
    /// - fetchAllTags 실패 시 tags = [] (폴백) — items 로드는 영향받지 않음.
    func loadFavorites() async {
        guard !hasLoaded else { return }

        phase = .loading
        do {
            let fetchedItems = try await favorites.listFavorites()
            items = fetchedItems
            hasLoaded = true
            phase = .done

            // items에 실제 존재하는 tagId 교집합만 칩으로 표시.
            // fetchAllTags 실패 시 tags = [] 로 degrade — items 로드 결과에 영향 없음.
            allTagsCache = (try? await tagPort.fetchAllTags()) ?? []
            recomputeTags()
        } catch {
            phase = .failed(FavoritesErrorMessage.loadFailed)
        }
    }

    /// 즐겨찾기 여부 확인.
    func isLiked(_ url: String) -> Bool {
        likedUrls.contains(url)
    }

    /// MVP9 M2: 해당 기사 URL의 퀴즈 완료 여부 확인.
    /// 즐겨찾기에 없는 기사는 false 반환.
    func isQuizCompleted(_ url: String) -> Bool {
        items.first { $0.url == url }?.quizCompleted ?? false
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
            recomputeTags()
        } catch {
            operationError = errorMessage(from: error)
        }
    }

    /// MVP10 M1: 퀴즈 완료 마킹 — 서버 업데이트 + items 로컬 갱신.
    /// items 갱신으로 isQuizCompleted() 즉시 반영 → 버튼 전환 + 배지 표시.
    func markQuizCompleted(url: String) async {
        operationError = nil
        do {
            try await favorites.markQuizCompleted(url: url)
            items = items.map { item in
                guard item.url == url else { return item }
                return FavoriteItem(
                    id: item.id,
                    userId: item.userId,
                    title: item.title,
                    url: item.url,
                    snippet: item.snippet,
                    source: item.source,
                    publishedAt: item.publishedAt,
                    tagId: item.tagId,
                    summary: item.summary,
                    insight: item.insight,
                    likedAt: item.likedAt,
                    createdAt: item.createdAt,
                    imageUrl: item.imageUrl,
                    quizCompleted: true
                )
            }
        } catch {
            operationError = "퀴즈 완료 처리에 실패했습니다."
        }
    }

    /// DELETE /me/favorites → items에서 제거.
    /// 실패 시 phase 대신 operationError에 기록 — 목록 화면 유지.
    func removeFavorite(url: String) async {
        operationError = nil
        do {
            try await favorites.deleteFavorite(url: url)
            items = items.filter { $0.url != url }
            recomputeTags()
        } catch {
            operationError = FavoritesErrorMessage.removeFailed
        }
    }

    // MARK: - Private

    /// ST1 BUG-D: items 변경 후 tags를 allTagsCache 기반으로 재계산.
    /// addFavorite / removeFavorite 성공 후 호출.
    private func recomputeTags() {
        let favTagIds = Set(items.compactMap(\.tagId))
        tags = allTagsCache.filter { favTagIds.contains($0.id) }
    }

    /// ST2 BUG-E: removeFavorite 후 selectedTagId를 nil로 초기화해야 하는지 판단.
    /// - Parameters:
    ///   - remaining: 삭제 후 남은 즐겨찾기 아이템 목록
    ///   - current: 현재 선택된 tagId (nil이면 전체 탭)
    /// - Returns: nil로 초기화해야 하면 true
    static func shouldResetTagId(remaining: [FavoriteItem], current: UUID?) -> Bool {
        guard let tagId = current else { return false }
        return !remaining.contains { $0.tagId == tagId }
    }

    private func errorMessage(from error: Error) -> String {
        if let favError = error as? APIFavoritesError, favError == .conflict {
            return FavoritesErrorMessage.conflict
        }
        return FavoritesErrorMessage.addFailed
    }
}
