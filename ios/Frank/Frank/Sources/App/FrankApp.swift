import SwiftUI
import Supabase

@main
struct FrankApp: App {
    @State private var dependencies: AppDependencies
    @State private var authFeature: AuthFeature

    init() {
        Log.app.notice("FrankApp launched")
        let deps = AppDependencies.live()
        _dependencies = State(initialValue: deps)
        _authFeature = State(initialValue: AuthFeature(auth: deps.auth))
    }

    var body: some Scene {
        WindowGroup {
            RootView(feature: authFeature, dependencies: dependencies)
        }
    }
}

struct RootView: View {
    let feature: AuthFeature
    let dependencies: AppDependencies

    var body: some View {
        switch feature.state {
        case .checkingSession, .authenticating:
            SplashView(feature: feature)
        case .unauthenticated:
            LoginView(feature: feature)
        case .authenticated(let profile):
            if profile.onboardingCompleted {
                ContentPlaceholderView(profile: profile, feature: feature)
            } else {
                OnboardingContainerView(
                    dependencies: dependencies,
                    authFeature: feature
                )
            }
        }
    }
}

struct OnboardingContainerView: View {
    let dependencies: AppDependencies
    let authFeature: AuthFeature
    @State private var onboardingFeature: OnboardingFeature?

    var body: some View {
        if let feature = onboardingFeature {
            OnboardingView(feature: feature)
        } else {
            ProgressView()
                .onAppear {
                    onboardingFeature = OnboardingFeature(
                        tag: dependencies.tag,
                        auth: dependencies.auth,
                        onComplete: {
                            Task {
                                await authFeature.send(.checkSession)
                            }
                        }
                    )
                }
        }
    }
}

// 임시 피드 플레이스홀더 (M4에서 교체)
struct ContentPlaceholderView: View {
    let profile: Profile
    let feature: AuthFeature

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 48))
                .foregroundStyle(.green)
            Text("로그인 완료")
                .font(.title2)
                .fontWeight(.semibold)
            Text(profile.email)
                .foregroundStyle(.secondary)
            Button("로그아웃") {
                Task {
                    await feature.send(.signOut)
                }
            }
            .buttonStyle(.bordered)
        }
    }
}
