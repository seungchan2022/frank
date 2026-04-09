import Foundation

/// 앱 세션 내 요약 결과 캐시.
/// url absoluteString → SummaryResult 매핑.
/// 앱 재시작 시 초기화됨.
@MainActor
final class SummarySessionCache {
    static let shared = SummarySessionCache()
    private var data: [String: SummaryResult] = [:]

    func get(_ url: String) -> SummaryResult? { data[url] }

    func set(_ url: String, _ result: SummaryResult) { data[url] = result }
}
