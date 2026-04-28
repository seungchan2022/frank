import Foundation

/// MVP11 M4: 오답 탭 태그 필터 순수 함수 헬퍼.
///
/// 뷰 레벨 computed 로직을 독립 테스트 가능한 정적 함수로 추출.
///
/// MVP13 M2: favorites 브릿지(tagMap) 완전 제거.
/// WrongAnswer.tagId 직접 비교로 단순화.
enum WrongAnswerTagFilter {

    /// 오답 목록을 선택된 tagId 기준으로 필터링.
    ///
    /// - Parameters:
    ///   - items: 전체 오답 목록
    ///   - selectedTagId: 선택된 태그. nil이면 전체 반환.
    /// - Returns:
    ///   - selectedTagId == nil: 전체 반환
    ///   - selectedTagId != nil: wa.tagId == selectedTagId 인 항목만 반환
    static func filter(
        items: [WrongAnswer],
        selectedTagId: UUID?
    ) -> [WrongAnswer] {
        guard let tagId = selectedTagId else { return items }
        return items.filter { $0.tagId == tagId }
    }
}
