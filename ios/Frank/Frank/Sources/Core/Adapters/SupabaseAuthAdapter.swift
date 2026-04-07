import Foundation
import Supabase
import Auth

/// Supabase Auth SDK를 사용한 AuthPort 구현체.
///
/// **데이터 출처 분리**:
/// - 인증(signIn/signUp/signOut/session) → Supabase SDK
/// - **Profile(displayName, onboardingCompleted) → Rust API `profiles` 테이블** (ProfileAPI 위임)
///
/// 이전 구현은 Supabase user_metadata에서 profile을 읽어 server profiles 테이블과 분리되었음.
/// M3에서 두 출처를 server profiles 테이블로 통일.
struct SupabaseAuthAdapter: AuthPort {
    private let client: SupabaseClient
    private let serverURL: URL
    private let session: URLSession

    init(
        client: SupabaseClient,
        serverConfig: ServerConfig = .live,
        session: URLSession = .shared
    ) {
        self.client = client
        self.serverURL = serverConfig.url
        self.session = session
    }

    func signIn(email: String, password: String) async throws -> Profile {
        let supabaseSession = try await client.auth.signIn(
            email: email,
            password: password
        )
        return try await fetchServerProfile(token: supabaseSession.accessToken, fallbackUser: supabaseSession.user)
    }

    func signUp(email: String, password: String) async throws -> Profile? {
        let response = try await client.auth.signUp(
            email: email,
            password: password
        )
        switch response {
        case .session(let supabaseSession):
            return try await fetchServerProfile(
                token: supabaseSession.accessToken,
                fallbackUser: supabaseSession.user
            )
        case .user:
            // 이메일 확인이 필요한 경우 세션 없이 user만 반환됨
            return nil
        }
    }

    func signInWithApple(idToken: String, rawNonce: String) async throws -> Profile {
        let supabaseSession = try await client.auth.signInWithIdToken(
            credentials: OpenIDConnectCredentials(
                provider: .apple,
                idToken: idToken,
                nonce: rawNonce
            )
        )
        return try await fetchServerProfile(
            token: supabaseSession.accessToken,
            fallbackUser: supabaseSession.user
        )
    }

    func signOut() async throws {
        try await client.auth.signOut()
    }

    func currentSession() async throws -> Profile? {
        guard let supabaseSession = try? await client.auth.session else {
            return nil
        }
        return try await fetchServerProfile(
            token: supabaseSession.accessToken,
            fallbackUser: supabaseSession.user
        )
    }

    /// Rust API `PUT /api/me/profile` 호출로 onboarding 완료 처리.
    /// (이전 구현은 Supabase user_metadata 직접 쓰기 — server profiles 테이블과 분리되어 있어 정정됨)
    func updateOnboardingCompleted() async throws -> Profile {
        let token = try await getAccessToken()
        return try await ProfileAPI.updateProfile(
            token: token,
            onboardingCompleted: true,
            serverURL: serverURL,
            session: session
        )
    }

    func getAccessToken() async throws -> String {
        let supabaseSession = try await client.auth.session
        return supabaseSession.accessToken
    }

    // MARK: - Private

    /// 진실의 원천(server `profiles` 테이블)에서 profile fetch.
    /// server 호출 실패 시(예: 신규 가입 직후 server 동기화 race) Supabase user 기반 minimal profile fallback.
    /// fallback의 onboardingCompleted는 항상 false (보수적 — 사용자가 onboarding 거치도록 유도).
    private func fetchServerProfile(token: String, fallbackUser: Auth.User) async throws -> Profile {
        do {
            return try await ProfileAPI.fetchProfile(
                token: token,
                serverURL: serverURL,
                session: session
            )
        } catch ProfileAPIError.notFound {
            // 첫 가입 시 server에 profile row가 아직 없을 수 있음 → fallback
            return Profile(
                id: fallbackUser.id,
                displayName: nil,
                onboardingCompleted: false
            )
        }
    }
}
