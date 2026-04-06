import Testing
import Foundation
@testable import Frank

@Suite("TagChipBarView Tests")
@MainActor
struct TagChipBarViewTests {

    // MARK: - Selection State

    @Test("selectedTagId가 nil이면 '전체'가 선택 상태")
    func allSelectedWhenNilTagId() {
        let tags = [
            Frank.Tag(id: UUID(), name: "AI", category: "ai"),
            Frank.Tag(id: UUID(), name: "iOS", category: "ios"),
        ]
        let view = TagChipBarView(
            tags: tags,
            selectedTagId: nil,
            onSelect: { _ in }
        )
        #expect(view.selectedTagId == nil)
        #expect(view.tags.count == 2)
    }

    @Test("특정 태그 ID가 선택되면 해당 태그가 선택 상태")
    func specificTagSelected() {
        let selectedId = UUID()
        let tags = [
            Frank.Tag(id: selectedId, name: "AI", category: "ai"),
            Frank.Tag(id: UUID(), name: "iOS", category: "ios"),
        ]
        let view = TagChipBarView(
            tags: tags,
            selectedTagId: selectedId,
            onSelect: { _ in }
        )
        #expect(view.selectedTagId == selectedId)
    }

    // MARK: - onSelect Callback

    @Test("'전체' 탭 시 onSelect(nil) 호출")
    func onSelectCalledWithNilForAll() {
        var receivedId: UUID?? = .none
        let tags = [Frank.Tag(id: UUID(), name: "AI", category: "ai")]
        let view = TagChipBarView(
            tags: tags,
            selectedTagId: tags[0].id,
            onSelect: { id in receivedId = .some(id) }
        )
        view.onSelect(nil)
        #expect(receivedId == .some(nil))
    }

    @Test("태그 탭 시 onSelect(tagId) 호출")
    func onSelectCalledWithTagId() {
        var receivedId: UUID?? = .none
        let tagId = UUID()
        let tags = [Frank.Tag(id: tagId, name: "AI", category: "ai")]
        let view = TagChipBarView(
            tags: tags,
            selectedTagId: nil,
            onSelect: { id in receivedId = .some(id) }
        )
        view.onSelect(tagId)
        #expect(receivedId == .some(tagId))
    }

    // MARK: - Tags List

    @Test("빈 태그 목록에서도 '전체' 버튼은 존재")
    func emptyTagsStillHasAllButton() {
        let view = TagChipBarView(
            tags: [],
            selectedTagId: nil,
            onSelect: { _ in }
        )
        #expect(view.tags.isEmpty)
        #expect(view.selectedTagId == nil)
    }

    @Test("태그 순서가 입력 순서대로 유지")
    func tagsOrderPreserved() {
        let tag1 = Frank.Tag(id: UUID(), name: "A", category: "a")
        let tag2 = Frank.Tag(id: UUID(), name: "B", category: "b")
        let tag3 = Frank.Tag(id: UUID(), name: "C", category: "c")
        let view = TagChipBarView(
            tags: [tag1, tag2, tag3],
            selectedTagId: nil,
            onSelect: { _ in }
        )
        #expect(view.tags[0].name == "A")
        #expect(view.tags[1].name == "B")
        #expect(view.tags[2].name == "C")
    }
}
