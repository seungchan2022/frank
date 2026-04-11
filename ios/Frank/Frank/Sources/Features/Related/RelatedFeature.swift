import Foundation
import Observation

/// MVP7 M3: RelatedFeature — 연관 기사 목록 상태 관리.
///
/// ArticleDetailView에서 지역 인스턴스로 생성하여 사용.
/// .task { await relatedFeature.load(title:snippet:) } 로 트리거.
@Observable
@MainActor
final class RelatedFeature {

    // MARK: - State

    /// 연관 기사 목록.
    private(set) var items: [FeedItem] = []

    /// 로딩 중 여부.
    private(set) var isLoading = false

    /// 에러 메시지 (nil = 에러 없음).
    private(set) var errorMessage: String? = nil

    // MARK: - Dependencies

    private let related: any RelatedPort

    // MARK: - Init

    init(related: any RelatedPort) {
        self.related = related
    }

    // MARK: - Actions

    /// 연관 기사 로드.
    /// - 호출마다 items 초기화 후 새 결과로 교체.
    func load(title: String, snippet: String?) async {
        isLoading = true
        errorMessage = nil
        items = []

        do {
            let result = try await related.fetchRelated(title: title, snippet: snippet)
            items = result
        } catch {
            errorMessage = "연관 기사를 불러오지 못했습니다."
        }

        isLoading = false
    }
}
