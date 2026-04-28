import Foundation

extension Notification.Name {
    /// 오답이 서버에 저장된 직후 발송. FavoritesView 오답 탭 자동 리로드용.
    static let wrongAnswerSaved = Notification.Name("wrongAnswerSaved")
}
