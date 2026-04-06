import Foundation
import Supabase
import Auth

struct SupabaseAuthAdapter: AuthPort {
    private let client: SupabaseClient

    init(client: SupabaseClient) {
        self.client = client
    }

    func signIn(email: String, password: String) async throws -> Profile {
        let session = try await client.auth.signIn(
            email: email,
            password: password
        )
        return mapToProfile(user: session.user)
    }

    func signUp(email: String, password: String) async throws -> Profile? {
        let response = try await client.auth.signUp(
            email: email,
            password: password
        )
        switch response {
        case .session(let session):
            return mapToProfile(user: session.user)
        case .user:
            // 이메일 확인이 필요한 경우 세션 없이 user만 반환됨
            return nil
        }
    }

    func signInWithApple(idToken: String, rawNonce: String) async throws -> Profile {
        let session = try await client.auth.signInWithIdToken(
            credentials: OpenIDConnectCredentials(
                provider: .apple,
                idToken: idToken,
                nonce: rawNonce
            )
        )
        return mapToProfile(user: session.user)
    }

    func signOut() async throws {
        try await client.auth.signOut()
    }

    func currentSession() async throws -> Profile? {
        guard let session = try? await client.auth.session else {
            return nil
        }
        return mapToProfile(user: session.user)
    }

    func updateOnboardingCompleted() async throws -> Profile {
        let user = try await client.auth.update(
            user: UserAttributes(data: ["onboarding_completed": .bool(true)])
        )
        return mapToProfile(user: user)
    }

    func getAccessToken() async throws -> String {
        let session = try await client.auth.session
        return session.accessToken
    }

    // MARK: - Private

    private func mapToProfile(user: Auth.User) -> Profile {
        Profile(
            id: user.id,
            email: user.email ?? "",
            onboardingCompleted: user.userMetadata["onboarding_completed"]?.boolValue ?? false
        )
    }
}
