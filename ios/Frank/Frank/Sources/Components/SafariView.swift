import SwiftUI
import SafariServices

/// 앱 내 Safari 뷰 — SFSafariViewController 래퍼.
/// ArticleDetailView 원문 보기에서 .sheet으로 표시.
struct SafariView: UIViewControllerRepresentable {
    let url: URL

    func makeUIViewController(context: Context) -> SFSafariViewController {
        SFSafariViewController(url: url)
    }

    func updateUIViewController(_ uiViewController: SFSafariViewController, context: Context) {}
}
