import Foundation

/// In-memory RewritePort 구현 — FRANK_USE_MOCK=1 모드 전용.
/// occupation 미설정 시 APIRewriteError.occupationRequired 에러 반환.
struct MockRewriteAdapter: RewritePort {
    private let auth: any AuthPort

    init(auth: any AuthPort = MockAuthAdapter()) {
        self.auth = auth
    }

    func rewrite(url: String, title: String) async throws -> RewriteResult {
        let profile = try await auth.currentProfile()
        guard let occupation = profile?.occupation else {
            throw APIRewriteError.occupationRequired
        }
        // 600ms 지연으로 실제 API 호출 시뮬레이션
        try await Task.sleep(for: .milliseconds(600))
        return RewriteResult(
            rewrite: "Mock 재작성 (\(occupation) 시각): \(title)에 대한 맞춤 요약입니다."
        )
    }
}
