import SwiftUI

struct EmailSignInSheet: View {
    let feature: AuthFeature

    @Environment(\.dismiss) private var dismiss
    @State private var email = ""
    @State private var password = ""
    @State private var isSignUp = false

    private var isFormValid: Bool {
        !email.isEmpty && !password.isEmpty && password.count >= 6
    }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("이메일", text: $email)
                        .textContentType(.emailAddress)
                        .keyboardType(.emailAddress)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)

                    SecureField("비밀번호 (6자 이상)", text: $password)
                        .textContentType(isSignUp ? .newPassword : .password)
                }

                Section {
                    Button {
                        submit()
                    } label: {
                        HStack {
                            Spacer()
                            if feature.state == .authenticating {
                                ProgressView()
                            } else {
                                Text(isSignUp ? "회원가입" : "로그인")
                                    .fontWeight(.semibold)
                            }
                            Spacer()
                        }
                    }
                    .disabled(!isFormValid || feature.state == .authenticating)
                }

                if let error = feature.error {
                    Section {
                        Text(error.koreanDescription)
                            .font(.caption)
                            .foregroundStyle(.red)
                            .accessibilityAddTraits(.isStaticText)
                    }
                }

                if let message = feature.confirmationMessage {
                    Section {
                        Label(message, systemImage: "envelope.badge")
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .navigationTitle(isSignUp ? "회원가입" : "이메일 로그인")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("닫기") { dismiss() }
                }
                ToolbarItem(placement: .bottomBar) {
                    Button {
                        isSignUp.toggle()
                    } label: {
                        Text(isSignUp ? "이미 계정이 있나요? 로그인" : "계정이 없나요? 회원가입")
                            .font(.footnote)
                    }
                }
            }
            .onChange(of: feature.state) { _, newValue in
                if case .authenticated = newValue {
                    dismiss()
                }
            }
        }
        .presentationDetents([.medium])
    }

    private func submit() {
        Task {
            if isSignUp {
                await feature.send(.signUp(email: email, password: password))
            } else {
                await feature.send(.signInWithEmail(email: email, password: password))
            }
        }
    }
}
