import Foundation

protocol AuthPort: Sendable {
    func signIn(email: String, password: String) async throws -> Profile
    func signUp(email: String, password: String) async throws -> Profile?
    func signInWithApple(idToken: String, rawNonce: String) async throws -> Profile
    func signOut() async throws
    func currentSession() async throws -> Profile?
    func updateOnboardingCompleted() async throws -> Profile
    func getAccessToken() async throws -> String
}
