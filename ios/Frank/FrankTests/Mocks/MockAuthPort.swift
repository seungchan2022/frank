import Foundation
@testable import Frank

final class MockAuthPort: AuthPort, @unchecked Sendable {
    var signInResult: Result<Profile, Error> = .success(
        Profile(id: UUID(), email: "test@example.com", onboardingCompleted: false)
    )
    var signUpResult: Result<Profile, Error> = .success(
        Profile(id: UUID(), email: "test@example.com", onboardingCompleted: false)
    )
    var signOutError: Error?
    var currentSessionResult: Profile?

    var signInCallCount = 0
    var signUpCallCount = 0
    var signOutCallCount = 0

    func signIn(email: String, password: String) async throws -> Profile {
        signInCallCount += 1
        return try signInResult.get()
    }

    func signUp(email: String, password: String) async throws -> Profile {
        signUpCallCount += 1
        return try signUpResult.get()
    }

    func signOut() async throws {
        signOutCallCount += 1
        if let error = signOutError { throw error }
    }

    func currentSession() async throws -> Profile? {
        currentSessionResult
    }
}
