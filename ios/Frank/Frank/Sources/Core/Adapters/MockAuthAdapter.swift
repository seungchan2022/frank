import Foundation

/// In-memory AuthPort 구현.
///
/// `scenario`에 따라 동작을 전환한다:
/// - `nil` (기본): 항상 fixture 사용자로 로그인된 상태
/// - `"logged_out"`: `currentSession()` → nil (로그인 화면 노출)
/// - `"new_user"`: `onboardingCompleted=false` 프로필로 로그인 (온보딩 화면 노출)
actor MockAuthAdapter: AuthPort {
    private var profile: Profile
    private let mockToken = "mock-access-token"
    private let scenario: String?

    init(profile: Profile = MockFixtures.profile, scenario: String? = nil) {
        self.profile = profile
        self.scenario = scenario
    }

    func signIn(email: String, password: String) async throws -> Profile {
        profile
    }

    func signUp(email: String, password: String) async throws -> Profile? {
        profile
    }

    func signInWithApple(idToken: String, rawNonce: String) async throws -> Profile {
        profile
    }

    func signOut() async throws {}

    func currentSession() async throws -> Profile? {
        if scenario == "logged_out" { return nil }
        return profile
    }

    func updateOnboardingCompleted() async throws -> Profile {
        profile = Profile(
            id: profile.id,
            displayName: profile.displayName,
            onboardingCompleted: true
        )
        return profile
    }

    func getAccessToken() async throws -> String {
        mockToken
    }
}
