import Foundation
@testable import Frank

final class MockAuthPort: AuthPort, @unchecked Sendable {
    var signInResult: Result<Profile, Error> = .success(
        Profile(id: UUID(), displayName: "test", onboardingCompleted: false)
    )
    var signUpResult: Result<Profile?, Error> = .success(
        Profile(id: UUID(), displayName: "test", onboardingCompleted: false)
    )
    var signInWithAppleResult: Result<Profile, Error> = .success(
        Profile(id: UUID(), displayName: "apple", onboardingCompleted: false)
    )
    var signOutError: Error?
    var currentSessionResult: Profile?
    var updateOnboardingCompletedResult: Result<Profile, Error> = .success(
        Profile(id: UUID(), displayName: "test", onboardingCompleted: true)
    )

    var accessToken: String = "mock-token"
    var getAccessTokenError: Error?

    var signInCallCount = 0
    var signUpCallCount = 0
    var signInWithAppleCallCount = 0
    var signOutCallCount = 0
    var updateOnboardingCompletedCallCount = 0
    var getAccessTokenCallCount = 0

    func signIn(email: String, password: String) async throws -> Profile {
        signInCallCount += 1
        return try signInResult.get()
    }

    func signUp(email: String, password: String) async throws -> Profile? {
        signUpCallCount += 1
        return try signUpResult.get()
    }

    func signInWithApple(idToken: String, rawNonce: String) async throws -> Profile {
        signInWithAppleCallCount += 1
        return try signInWithAppleResult.get()
    }

    func signOut() async throws {
        signOutCallCount += 1
        if let error = signOutError { throw error }
    }

    func currentSession() async throws -> Profile? {
        currentSessionResult
    }

    func updateOnboardingCompleted() async throws -> Profile {
        updateOnboardingCompletedCallCount += 1
        return try updateOnboardingCompletedResult.get()
    }

    func getAccessToken() async throws -> String {
        getAccessTokenCallCount += 1
        if let error = getAccessTokenError { throw error }
        return accessToken
    }
}
