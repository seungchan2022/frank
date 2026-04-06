import SwiftUI

struct TagManagementView: View {
    let feature: SettingsFeature
    var onTagsSaved: (() -> Void)?
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(spacing: 0) {
            ScrollView {
                VStack(alignment: .leading, spacing: 24) {
                    headerSection
                    tagChipsSection
                }
                .padding(.horizontal, 20)
                .padding(.top, 20)
            }

            Spacer()

            errorAndSaveSection
        }
        .navigationTitle("태그 관리")
        .navigationBarTitleDisplayMode(.inline)
        .allowsHitTesting(!feature.isSaving)
    }

    // MARK: - Sections

    private var headerSection: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("관심 키워드를 편집하세요")
                .font(.headline)
            Text("최소 1개 이상 선택해야 합니다")
                .font(.subheadline)
                .foregroundStyle(.secondary)
        }
    }

    private var tagChipsSection: some View {
        FlowLayout(spacing: 10) {
            ForEach(feature.tags) { tag in
                TagChipView(
                    tag: tag,
                    isSelected: feature.selectedIds.contains(tag.id),
                    onTap: {
                        Task {
                            await feature.send(.toggleTag(tag.id))
                        }
                    }
                )
            }
        }
    }

    private var errorAndSaveSection: some View {
        VStack(spacing: 8) {
            if let errorMessage = feature.errorMessage {
                Text(errorMessage)
                    .font(.caption)
                    .foregroundStyle(.red)
            }

            Button {
                Task {
                    await feature.send(.saveTags)
                    if feature.tagsChanged {
                        onTagsSaved?()
                        dismiss()
                    }
                }
            } label: {
                Group {
                    if feature.isSaving {
                        ProgressView()
                            .tint(.white)
                    } else {
                        Text("저장")
                            .fontWeight(.semibold)
                    }
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 16)
            }
            .buttonStyle(.borderedProminent)
            .disabled(!feature.canSave)
            .padding(.horizontal, 20)
            .padding(.bottom, 8)
        }
    }
}
