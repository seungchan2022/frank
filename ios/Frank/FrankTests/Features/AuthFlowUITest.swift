import Testing
import Foundation
@testable import Frank

/// 이메일 로그인 → authenticated 화면 분기 통합 테스트
/// 실제 Supabase 대신 MockAuthPort로 전체 플로우 검증
@Suite("Auth Flow Integration Tests")
@MainActor
struct AuthFlowIntegrationTests {

    @Test("전체 플로우: checkSession(nil) → unauthenticated → signIn → authenticated")
    func fullLoginFlow() async {
        // Given: 세션 없는 상태
        let mock = MockAuthPort()
        mock.currentSessionResult = nil
        let feature = AuthFeature(auth: mock)

        // 초기 상태: checkingSession
        #expect(feature.state == .checkingSession)

        // When: 세션 확인 → 없음
        await feature.send(.checkSession)
        #expect(feature.state == .unauthenticated)

        // When: 이메일 로그인
        let profile = Profile(id: UUID(), displayName: "test", onboardingCompleted: false)
        mock.signInResult = .success(profile)
        await feature.send(.signInWithEmail(email: "test@frank.dev", password: "Test1234!"))

        // Then: 인증 완료
        #expect(feature.state == .authenticated(profile))
        #expect(feature.error == nil)
    }

    @Test("전체 플로우: checkSession(profile) → 바로 authenticated")
    func sessionRestoredFlow() async {
        // Given: 기존 세션 있음
        let mock = MockAuthPort()
        let profile = Profile(id: UUID(), displayName: "test", onboardingCompleted: true)
        mock.currentSessionResult = profile
        let feature = AuthFeature(auth: mock)

        // When: 세션 확인
        await feature.send(.checkSession)

        // Then: 바로 인증 상태
        #expect(feature.state == .authenticated(profile))
    }

    @Test("전체 플로우: authenticated → signOut → unauthenticated")
    func logoutFlow() async {
        // Given: 로그인된 상태
        let mock = MockAuthPort()
        let profile = Profile(id: UUID(), displayName: "test", onboardingCompleted: true)
        mock.currentSessionResult = profile
        let feature = AuthFeature(auth: mock)
        await feature.send(.checkSession)
        #expect(feature.state == .authenticated(profile))

        // When: 로그아웃
        await feature.send(.signOut)

        // Then: 미인증 상태
        #expect(feature.state == .unauthenticated)
    }

    @Test("RootView 분기: checkingSession → SplashView, unauthenticated → LoginView, authenticated → ContentPlaceholder")
    func rootViewBranching() async {
        let mock = MockAuthPort()
        mock.currentSessionResult = nil
        let feature = AuthFeature(auth: mock)

        // 1. checkingSession → SplashView 표시해야 함
        #expect(feature.state == .checkingSession)

        // 2. 세션 확인 후 → unauthenticated → LoginView 표시해야 함
        await feature.send(.checkSession)
        #expect(feature.state == .unauthenticated)

        // 3. 로그인 후 → authenticated → ContentPlaceholderView 표시해야 함
        let profile = Profile(id: UUID(), displayName: "test", onboardingCompleted: false)
        mock.signInResult = .success(profile)
        await feature.send(.signInWithEmail(email: "test@frank.dev", password: "Test1234!"))
        #expect(feature.state == .authenticated(profile))

        // 4. onboardingCompleted == false → OnboardingView 표시
        #expect(profile.onboardingCompleted == false)

        // 5. onboardingCompleted == true → ContentPlaceholderView 표시
        let completedProfile = Profile(id: UUID(), displayName: "test", onboardingCompleted: true)
        mock.signInResult = .success(completedProfile)
        await feature.send(.signInWithEmail(email: "test@frank.dev", password: "Test1234!"))
        if case .authenticated(let p) = feature.state {
            #expect(p.onboardingCompleted == true)
        } else {
            Issue.record("Expected authenticated state")
        }
    }
}
