import SwiftUI

struct OnboardingView: View {
    let feature: OnboardingFeature

    var body: some View {
        content
            .task {
                await feature.send(.loadTags)
            }
    }

    @ViewBuilder
    private var content: some View {
        switch feature.state {
        case .loading:
            ProgressView()
        case let .loaded(tags, selectedIds):
            if tags.isEmpty && feature.errorMessage != nil {
                errorView(message: feature.errorMessage ?? "")
            } else {
                tagSelectionView(tags: tags, selectedIds: selectedIds)
            }
        }
    }

    private func tagSelectionView(tags: [Tag], selectedIds: Set<UUID>) -> some View {
        VStack(spacing: 0) {
            ScrollView {
                VStack(alignment: .leading, spacing: 24) {
                    headerSection
                    tagChipsSection(tags: tags, selectedIds: selectedIds)
                }
                .padding(.horizontal, 20)
                .padding(.top, 40)
            }

            Spacer()

            ctaSection
        }
        .allowsHitTesting(!feature.isSaving)
    }

    private var headerSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("관심 키워드를 선택하세요")
                .font(.title)
                .fontWeight(.bold)
            Text("선택한 키워드 기반으로 뉴스를 모아드려요")
                .font(.subheadline)
                .foregroundStyle(.secondary)
        }
    }

    private func tagChipsSection(tags: [Tag], selectedIds: Set<UUID>) -> some View {
        FlowLayout(spacing: 10) {
            ForEach(tags) { tag in
                TagChipView(
                    tag: tag,
                    isSelected: selectedIds.contains(tag.id),
                    onTap: {
                        Task {
                            await feature.send(.toggleTag(id: tag.id))
                        }
                    }
                )
            }
        }
    }

    private var ctaSection: some View {
        VStack(spacing: 8) {
            if let errorMessage = feature.errorMessage {
                Text(errorMessage)
                    .font(.caption)
                    .foregroundStyle(.red)
            }

            Button {
                Task {
                    await feature.send(.complete)
                }
            } label: {
                Group {
                    if feature.isSaving {
                        ProgressView()
                            .tint(.white)
                    } else {
                        Text("시작하기")
                            .fontWeight(.semibold)
                    }
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 16)
            }
            .buttonStyle(.borderedProminent)
            .disabled(!feature.canComplete)
            .padding(.horizontal, 20)
            .padding(.bottom, 8)
        }
    }

    private func errorView(message: String) -> some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 40))
                .foregroundStyle(.secondary)
            Text(message)
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
            Button("다시 시도") {
                Task {
                    await feature.send(.loadTags)
                }
            }
            .buttonStyle(.bordered)
        }
        .padding()
    }
}
