import Foundation

protocol AuthPort: Sendable {
    func signIn(email: String, password: String) async throws -> Profile
    func signUp(email: String, password: String) async throws -> Profile?
    func signInWithApple(idToken: String, rawNonce: String) async throws -> Profile
    func signOut() async throws
    func currentSession() async throws -> Profile?
    func updateOnboardingCompleted() async throws -> Profile
    func getAccessToken() async throws -> String
    /// MVP15 M3: 직업 업데이트. nil = 삭제, 빈 문자열 = nil 처리(서버에서 trim).
    func updateOccupation(_ occupation: String?) async throws -> Profile
    /// MVP15 M3: 현재 프로필 조회 (occupation 포함).
    func currentProfile() async throws -> Profile?
}
