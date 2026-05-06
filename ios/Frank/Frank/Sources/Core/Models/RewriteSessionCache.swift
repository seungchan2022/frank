import Foundation

/// 앱 세션 내 재작성 결과 캐시.
/// url absoluteString → (RewriteResult, occupation?) 매핑.
/// MVP16 M3 (C2-bug): occupation도 함께 캐싱.
/// occupation 변경 후 재진입 시 occupation 불일치 → 버튼 재활성화.
/// 앱 재시작 시 초기화됨.
@MainActor
final class RewriteSessionCache {
    static let shared = RewriteSessionCache()

    private struct CacheEntry {
        let result: RewriteResult
        /// 재작성 당시의 occupation. nil = occupation 미설정 상태로 재작성.
        let occupation: String?
    }

    private var data: [String: CacheEntry] = [:]

    func get(_ url: String) -> RewriteResult? { data[url]?.result }

    /// occupation과 함께 캐시에서 조회. (result, occupation) 쌍 반환.
    func getWithOccupation(_ url: String) -> (result: RewriteResult, occupation: String?)? {
        guard let entry = data[url] else { return nil }
        return (entry.result, entry.occupation)
    }

    /// occupation과 함께 저장. occupation 없이 저장 시 nil (= 직업 미설정 상태로 재작성).
    func set(_ url: String, _ result: RewriteResult, occupation: String? = nil) {
        data[url] = CacheEntry(result: result, occupation: occupation)
    }
}
