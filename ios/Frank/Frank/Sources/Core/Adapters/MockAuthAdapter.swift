import Foundation

/// In-memory AuthPort 구현. 항상 fixture 사용자로 로그인된 상태.
actor MockAuthAdapter: AuthPort {
    private var profile: Profile
    private let mockToken = "mock-access-token"

    init(profile: Profile = MockFixtures.profile) {
        self.profile = profile
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
        profile
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
