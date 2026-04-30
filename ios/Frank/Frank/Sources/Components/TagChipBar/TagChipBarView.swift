import SwiftUI

struct TagChipBarView: View {
    let tags: [Tag]
    let selectedTagId: UUID?
    let onSelect: (UUID?) -> Void

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    allButton
                        .id("all")
                    ForEach(tags) { tag in
                        TagChipView(
                            tag: tag,
                            isSelected: selectedTagId == tag.id,
                            onTap: { onSelect(tag.id) }
                        )
                        .id(tag.id)
                    }
                }
                .padding(.horizontal, 16)
            }
            .onChange(of: selectedTagId) { _, newTagId in
                withAnimation(.easeInOut(duration: 0.25)) {
                    if let newTagId {
                        proxy.scrollTo(newTagId, anchor: .center)
                    } else {
                        proxy.scrollTo("all", anchor: .center)
                    }
                }
            }
        }
    }

    private var allButton: some View {
        Button(action: { onSelect(nil) }) {
            Text("전체")
                .tagChipStyle(isSelected: selectedTagId == nil)
        }
        .buttonStyle(.plain)
        .accessibilityLabel("전체")
        .accessibilityAddTraits(selectedTagId == nil ? .isSelected : [])
    }
}

#Preview("TagChipBarView - 전체 선택") {
    TagChipBarView(
        tags: [
            Tag(id: UUID(), name: "AI", category: "ai"),
            Tag(id: UUID(), name: "iOS", category: "ios"),
            Tag(id: UUID(), name: "Backend", category: "backend"),
            Tag(id: UUID(), name: "DevOps", category: "devops"),
            Tag(id: UUID(), name: "Frontend", category: "frontend"),
        ],
        selectedTagId: nil,
        onSelect: { _ in }
    )
}

#Preview("TagChipBarView - 태그 선택") {
    let selectedId = UUID()
    TagChipBarView(
        tags: [
            Tag(id: selectedId, name: "AI", category: "ai"),
            Tag(id: UUID(), name: "iOS", category: "ios"),
            Tag(id: UUID(), name: "Backend", category: "backend"),
        ],
        selectedTagId: selectedId,
        onSelect: { _ in }
    )
}
