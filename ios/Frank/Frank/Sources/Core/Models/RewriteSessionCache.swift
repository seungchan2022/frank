import Foundation

/// 앱 세션 내 재작성 결과 캐시.
/// url absoluteString → RewriteResult 매핑.
/// 앱 재시작 시 초기화됨.
@MainActor
final class RewriteSessionCache {
    static let shared = RewriteSessionCache()
    private var data: [String: RewriteResult] = [:]

    func get(_ url: String) -> RewriteResult? { data[url] }

    func set(_ url: String, _ result: RewriteResult) { data[url] = result }
}
