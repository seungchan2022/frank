import SwiftUI

/// MVP7 M3: RelatedArticlesView — 연관 기사 목록 뷰.
///
/// ArticleDetailView 하단에 삽입.
/// RelatedFeature를 주입받아 로딩/에러/빈 결과/목록 상태를 표시.
/// 각 아이템 탭 시 ArticleDetailView로 네비게이션 (지역 인스턴스 사용).
struct RelatedArticlesView: View {
    let feature: RelatedFeature
    let summarize: any SummarizePort
    let favoritesFeature: FavoritesFeature
    let likesFeature: LikesFeature
    /// 연관 기사에서 또 다른 연관 기사 탐색 시 사용할 포트.
    /// 무한 중첩 방지를 위해 MockRelatedAdapter(빈 결과) 전달 가능.
    let nextRelated: any RelatedPort

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("연관 기사")
                .font(.subheadline)
                .fontWeight(.bold)
                .foregroundStyle(.secondary)

            contentView
        }
    }

    // MARK: - Content

    @ViewBuilder
    private var contentView: some View {
        if feature.isLoading {
            loadingView
        } else if let errorMessage = feature.errorMessage {
            errorView(message: errorMessage)
        } else if feature.items.isEmpty {
            emptyView
        } else {
            itemsView
        }
    }

    private var loadingView: some View {
        HStack {
            ProgressView()
                .padding(.trailing, 4)
            Text("연관 기사 불러오는 중...")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.vertical, 8)
    }

    private func errorView(message: String) -> some View {
        Text(message)
            .font(.caption)
            .foregroundStyle(.red)
            .padding(.vertical, 4)
    }

    private var emptyView: some View {
        Text("연관 기사가 없습니다.")
            .font(.caption)
            .foregroundStyle(.secondary)
            .padding(.vertical, 4)
    }

    private var itemsView: some View {
        VStack(spacing: 0) {
            ForEach(feature.items) { item in
                NavigationLink {
                    ArticleDetailView(
                        feedItem: item,
                        summarize: summarize,
                        favoritesFeature: favoritesFeature,
                        likesFeature: likesFeature,
                        related: nextRelated
                    )
                } label: {
                    ArticleCardView(article: item)
                }
                .buttonStyle(.plain)

                if item.id != feature.items.last?.id {
                    Divider()
                }
            }
        }
    }
}
