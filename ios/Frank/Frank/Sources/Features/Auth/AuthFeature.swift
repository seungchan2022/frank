import Foundation
import Observation

enum AuthState: Equatable {
    case checkingSession
    case authenticating
    case authenticated(Profile)
    case unauthenticated
}

enum AuthAction {
    case checkSession
    case signInWithEmail(email: String, password: String)
    case signUp(email: String, password: String)
    case signInWithApple(idToken: String, rawNonce: String)
    case signOut
}

@Observable
@MainActor
final class AuthFeature {
    private(set) var state: AuthState = .checkingSession
    private(set) var error: Error?
    private(set) var confirmationMessage: String?

    private let auth: any AuthPort

    init(auth: any AuthPort) {
        self.auth = auth
    }

    func send(_ action: AuthAction) async {
        switch action {
        case .checkSession:
            await checkSession()
        case let .signInWithEmail(email, password):
            await signInWithEmail(email: email, password: password)
        case let .signUp(email, password):
            await signUp(email: email, password: password)
        case let .signInWithApple(idToken, rawNonce):
            await signInWithApple(idToken: idToken, rawNonce: rawNonce)
        case .signOut:
            await signOut()
        }
    }

    func clearError() {
        error = nil
    }

    // MARK: - Private

    private func checkSession() async {
        do {
            if let profile = try await auth.currentSession() {
                state = .authenticated(profile)
            } else {
                state = .unauthenticated
            }
        } catch {
            state = .unauthenticated
        }
    }

    private func signInWithEmail(email: String, password: String) async {
        state = .authenticating
        error = nil
        do {
            let profile = try await auth.signIn(email: email, password: password)
            state = .authenticated(profile)
        } catch {
            state = .unauthenticated
            self.error = error
        }
    }

    private func signUp(email: String, password: String) async {
        state = .authenticating
        error = nil
        confirmationMessage = nil
        do {
            if let profile = try await auth.signUp(email: email, password: password) {
                state = .authenticated(profile)
            } else {
                state = .unauthenticated
                confirmationMessage = "이메일을 확인해주세요. 인증 링크를 보냈습니다."
            }
        } catch {
            state = .unauthenticated
            self.error = error
        }
    }

    private func signInWithApple(idToken: String, rawNonce: String) async {
        state = .authenticating
        error = nil
        do {
            let profile = try await auth.signInWithApple(idToken: idToken, rawNonce: rawNonce)
            state = .authenticated(profile)
        } catch {
            state = .unauthenticated
            self.error = error
        }
    }

    private func signOut() async {
        do {
            try await auth.signOut()
            state = .unauthenticated
            error = nil
        } catch {
            state = .unauthenticated
            self.error = error
        }
    }
}
