import Foundation

/// MVP11 M4: 오답 탭 태그 필터 순수 함수 헬퍼.
///
/// 뷰 레벨 computed 로직을 독립 테스트 가능한 정적 함수로 추출.
/// FavoritesView의 wrongAnswerTagMap / filteredWrongAnswers 계산에 사용.
enum WrongAnswerTagFilter {

    /// FavoriteItem 배열에서 url → tagId 매핑 생성.
    /// tagId가 없는 FavoriteItem은 포함하지 않음.
    static func buildTagMap(from favoriteItems: [FavoriteItem]) -> [String: UUID] {
        var map: [String: UUID] = [:]
        for item in favoriteItems {
            if let tagId = item.tagId {
                map[item.url] = tagId
            }
        }
        return map
    }

    /// 오답 목록을 태그 맵과 선택된 tagId 기준으로 필터링.
    ///
    /// - Parameters:
    ///   - items: 전체 오답 목록
    ///   - tagMap: url → tagId 매핑 (favorites 기반)
    ///   - selectedTagId: 선택된 태그. nil이면 전체 반환.
    /// - Returns:
    ///   - selectedTagId == nil: 전체 반환
    ///   - selectedTagId != nil:
    ///     1. 해당 tagId의 url Set 추출
    ///     2. wrongAnswer.articleUrl이 해당 urlSet에 포함 → 표시
    ///     3. wrongAnswer.articleUrl이 tagMap에 아예 없음(favorites 미등록) → 항상 표시
    static func apply(
        items: [WrongAnswer],
        tagMap: [String: UUID],
        selectedTagId: UUID?
    ) -> [WrongAnswer] {
        guard let tagId = selectedTagId else { return items }

        let urlSet = Set(tagMap.compactMap { url, tid in tid == tagId ? url : nil })

        return items.filter { wrongAnswer in
            // favorites에 없는 오답은 항상 표시
            if tagMap[wrongAnswer.articleUrl] == nil { return true }
            // 선택된 태그의 url과 일치하는 오답만 표시
            return urlSet.contains(wrongAnswer.articleUrl)
        }
    }
}
