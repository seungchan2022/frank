import SwiftUI
import AuthenticationServices

struct LoginView: View {
    let feature: AuthFeature

    @State private var showEmailSheet = false
    @State private var currentNonce: String?

    var body: some View {
        VStack(spacing: 0) {
            Spacer()

            // 앱 로고/타이틀
            VStack(spacing: 12) {
                Image(systemName: "newspaper.fill")
                    .font(.system(size: 64))
                    .foregroundStyle(.primary)
                Text("Frank")
                    .font(.largeTitle)
                    .fontWeight(.bold)
                Text("나만의 AI 뉴스 스터디")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            // Apple 로그인 버튼
            VStack(spacing: 16) {
                SignInWithAppleButton(.signIn) { request in
                    guard let nonce = try? AppleSignInHelper.randomNonce() else { return }
                    currentNonce = nonce
                    request.requestedScopes = [.fullName, .email]
                    request.nonce = AppleSignInHelper.sha256(nonce)
                } onCompletion: { result in
                    handleAppleSignIn(result)
                }
                .signInWithAppleButtonStyle(.black)
                .frame(height: 50)

                // 이메일 로그인 링크
                Button {
                    showEmailSheet = true
                } label: {
                    Text("다른 방법으로 로그인")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }

                // 에러 메시지 (버튼 하단 인라인)
                if let error = feature.error {
                    Text(error.localizedDescription)
                        .font(.caption)
                        .foregroundStyle(.red)
                        .multilineTextAlignment(.center)
                        .accessibilityAddTraits(.isStaticText)
                }
            }
            .padding(.horizontal, 32)
            .padding(.bottom, 48)
        }
        .sheet(isPresented: $showEmailSheet) {
            EmailSignInSheet(feature: feature)
        }
    }

    private func handleAppleSignIn(_ result: Result<ASAuthorization, Error>) {
        switch result {
        case .success(let authorization):
            guard let appleCredential = authorization.credential as? ASAuthorizationAppleIDCredential,
                  let identityToken = appleCredential.identityToken,
                  let idTokenString = String(data: identityToken, encoding: .utf8),
                  let nonce = currentNonce
            else {
                return
            }
            Task {
                await feature.send(.signInWithApple(idToken: idTokenString, rawNonce: nonce))
            }
        case .failure(let error):
            // 사용자가 직접 취소한 경우 무시
            if let authError = error as? ASAuthorizationError, authError.code == .canceled {
                return
            }
            // 그 외 에러는 feature에 전달 (alert 표시)
            feature.send(error)
        }
    }
}
