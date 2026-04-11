import Foundation
import Observation

// MARK: - 에러 메시지 한국어 변환

extension Error {
    /// URLError 등 시스템 에러를 사용자 친화적 한국어 메시지로 변환.
    var koreanDescription: String {
        if let urlError = self as? URLError {
            switch urlError.code {
            case .cannotFindHost, .dnsLookupFailed:
                return "서버에 연결할 수 없습니다. 인터넷 연결을 확인하거나, 앱 서버가 실행 중인지 확인해주세요."
            case .notConnectedToInternet:
                return "인터넷에 연결되어 있지 않습니다. Wi-Fi 또는 셀룰러 데이터를 확인해주세요."
            case .timedOut:
                return "서버 응답 시간이 초과되었습니다. 잠시 후 다시 시도해주세요."
            case .cannotConnectToHost:
                return "서버에 접속할 수 없습니다. 서버가 실행 중인지 확인해주세요."
            case .networkConnectionLost:
                return "네트워크 연결이 끊어졌습니다. 다시 시도해주세요."
            default:
                return "네트워크 오류가 발생했습니다. (\(urlError.localizedDescription))"
            }
        }
        // 로그인 실패 (이메일/비밀번호 불일치) 등
        let desc = localizedDescription
        if desc.lowercased().contains("invalid login credentials") ||
           desc.lowercased().contains("invalid_credentials") {
            return "이메일 또는 비밀번호가 올바르지 않습니다."
        }
        if desc.lowercased().contains("email not confirmed") {
            return "이메일 인증을 완료해주세요. 받은 편지함을 확인하세요."
        }
        return desc
    }
}

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

    /// Apple 로그인 취소 외의 에러를 LoginView에서 직접 전달할 때 사용.
    func send(_ error: Error) {
        state = .unauthenticated
        self.error = error
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
            self.error = error
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
