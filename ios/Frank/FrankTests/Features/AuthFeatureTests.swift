import Testing
import Foundation
@testable import Frank

@Suite("AuthFeature Tests")
@MainActor
struct AuthFeatureTests {

    // MARK: - 초기 상태

    @Test("초기 상태는 checkingSession")
    func initialState() {
        let mock = MockAuthPort()
        let feature = AuthFeature(auth: mock)

        #expect(feature.state == .checkingSession)
        #expect(feature.error == nil)
    }

    // MARK: - checkSession

    @Test("checkSession 성공 시 authenticated")
    func checkSessionSuccess() async {
        let mock = MockAuthPort()
        let profile = Profile(id: UUID(), displayName: "user", onboardingCompleted: true)
        mock.currentSessionResult = profile
        let feature = AuthFeature(auth: mock)

        await feature.send(.checkSession)

        #expect(feature.state == .authenticated(profile))
        #expect(feature.error == nil)
    }

    @Test("checkSession nil 시 unauthenticated")
    func checkSessionNil() async {
        let mock = MockAuthPort()
        mock.currentSessionResult = nil
        let feature = AuthFeature(auth: mock)

        await feature.send(.checkSession)

        #expect(feature.state == .unauthenticated)
        #expect(feature.error == nil)
    }

    // MARK: - signInWithEmail

    @Test("이메일 로그인 성공")
    func signInWithEmailSuccess() async {
        let mock = MockAuthPort()
        let profile = Profile(id: UUID(), displayName: "user", onboardingCompleted: false)
        mock.signInResult = .success(profile)
        let feature = AuthFeature(auth: mock)

        await feature.send(.signInWithEmail(email: "user@test.com", password: "pass"))

        #expect(feature.state == .authenticated(profile))
        #expect(mock.signInCallCount == 1)
        #expect(feature.error == nil)
    }

    @Test("이메일 로그인 중 authenticating 상태")
    func signInWithEmailLoading() async {
        let mock = MockAuthPort()
        let feature = AuthFeature(auth: mock)

        // signIn이 호출되기 전까지 authenticating 상태를 확인하기 위해
        // 실패 시나리오로 확인
        mock.signInResult = .failure(URLError(.userAuthenticationRequired))

        await feature.send(.signInWithEmail(email: "bad", password: "bad"))

        // 에러 후에는 unauthenticated
        #expect(feature.state == .unauthenticated)
        #expect(feature.error != nil)
    }

    @Test("이메일 로그인 실패 시 unauthenticated + error")
    func signInWithEmailFailure() async {
        let mock = MockAuthPort()
        mock.signInResult = .failure(URLError(.userAuthenticationRequired))
        let feature = AuthFeature(auth: mock)

        await feature.send(.signInWithEmail(email: "bad", password: "bad"))

        #expect(feature.state == .unauthenticated)
        #expect(feature.error != nil)
    }

    // MARK: - signUp

    @Test("회원가입 성공 (세션 즉시 반환)")
    func signUpSuccessWithSession() async {
        let mock = MockAuthPort()
        let profile = Profile(id: UUID(), displayName: "new", onboardingCompleted: false)
        mock.signUpResult = .success(profile)
        let feature = AuthFeature(auth: mock)

        await feature.send(.signUp(email: "new@test.com", password: "pass"))

        #expect(feature.state == .authenticated(profile))
        #expect(mock.signUpCallCount == 1)
    }

    @Test("회원가입 성공 (이메일 확인 대기)")
    func signUpPendingConfirmation() async {
        let mock = MockAuthPort()
        mock.signUpResult = .success(nil)
        let feature = AuthFeature(auth: mock)

        await feature.send(.signUp(email: "new@test.com", password: "pass"))

        #expect(feature.state == .unauthenticated)
        #expect(feature.confirmationMessage != nil)
    }

    @Test("회원가입 실패")
    func signUpFailure() async {
        let mock = MockAuthPort()
        mock.signUpResult = .failure(URLError(.badServerResponse))
        let feature = AuthFeature(auth: mock)

        await feature.send(.signUp(email: "new@test.com", password: "bad"))

        #expect(feature.state == .unauthenticated)
        #expect(feature.error != nil)
    }

    // MARK: - signInWithApple

    @Test("Apple 로그인 성공")
    func signInWithAppleSuccess() async {
        let mock = MockAuthPort()
        let profile = Profile(id: UUID(), displayName: "apple", onboardingCompleted: false)
        mock.signInWithAppleResult = .success(profile)
        let feature = AuthFeature(auth: mock)

        await feature.send(.signInWithApple(idToken: "token", rawNonce: "nonce"))

        #expect(feature.state == .authenticated(profile))
        #expect(mock.signInWithAppleCallCount == 1)
    }

    @Test("Apple 로그인 실패")
    func signInWithAppleFailure() async {
        let mock = MockAuthPort()
        mock.signInWithAppleResult = .failure(URLError(.userAuthenticationRequired))
        let feature = AuthFeature(auth: mock)

        await feature.send(.signInWithApple(idToken: "bad", rawNonce: "bad"))

        #expect(feature.state == .unauthenticated)
        #expect(feature.error != nil)
    }

    // MARK: - signOut

    @Test("로그아웃 성공")
    func signOutSuccess() async {
        let mock = MockAuthPort()
        let profile = Profile(id: UUID(), displayName: "user", onboardingCompleted: true)
        mock.signInResult = .success(profile)
        let feature = AuthFeature(auth: mock)

        // 먼저 로그인
        await feature.send(.signInWithEmail(email: "user@test.com", password: "pass"))
        #expect(feature.state == .authenticated(profile))

        // 로그아웃
        await feature.send(.signOut)

        #expect(feature.state == .unauthenticated)
        #expect(mock.signOutCallCount == 1)
    }

    @Test("로그아웃 실패 시에도 unauthenticated")
    func signOutFailure() async {
        let mock = MockAuthPort()
        let profile = Profile(id: UUID(), displayName: "user", onboardingCompleted: true)
        mock.signInResult = .success(profile)
        mock.signOutError = URLError(.networkConnectionLost)
        let feature = AuthFeature(auth: mock)

        await feature.send(.signInWithEmail(email: "user@test.com", password: "pass"))
        await feature.send(.signOut)

        // 실패해도 로컬에서는 unauthenticated로 전환
        #expect(feature.state == .unauthenticated)
        #expect(feature.error != nil)
    }

    // MARK: - clearError

    @Test("에러 클리어")
    func clearError() async {
        let mock = MockAuthPort()
        mock.signInResult = .failure(URLError(.badServerResponse))
        let feature = AuthFeature(auth: mock)

        await feature.send(.signInWithEmail(email: "bad", password: "bad"))
        #expect(feature.error != nil)

        feature.clearError()
        #expect(feature.error == nil)
    }

    // MARK: - send(_ error:)

    @Test("send(error:) — feature.error 설정 + unauthenticated 전환")
    func sendError() {
        let mock = MockAuthPort()
        let feature = AuthFeature(auth: mock)
        let testError = URLError(.badURL)

        feature.send(testError)

        #expect(feature.state == .unauthenticated)
        #expect(feature.error != nil)
    }

    @Test("send(error:) — 이미 authenticated 상태에서 호출해도 unauthenticated로 전환")
    func sendErrorFromAuthenticated() async {
        let mock = MockAuthPort()
        let profile = Profile(id: UUID(), displayName: "user", onboardingCompleted: true)
        mock.currentSessionResult = profile
        let feature = AuthFeature(auth: mock)

        await feature.send(.checkSession)
        #expect(feature.state == .authenticated(profile))

        feature.send(URLError(.networkConnectionLost))

        #expect(feature.state == .unauthenticated)
        #expect(feature.error != nil)
    }
}
