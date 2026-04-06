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
                FeedContainerView(
                    dependencies: dependencies,
                    authFeature: feature
                )
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

struct FeedContainerView: View {
    let authFeature: AuthFeature
    let articlePort: any ArticlePort
    @State private var feedFeature: FeedFeature
    @State private var showSettings = false

    init(dependencies: AppDependencies, authFeature: AuthFeature) {
        self.authFeature = authFeature
        self.articlePort = dependencies.article
        self._feedFeature = State(initialValue: FeedFeature(
            article: dependencies.article,
            collect: dependencies.collect,
            tag: dependencies.tag
        ))
    }

    var body: some View {
        FeedView(feature: feedFeature, articlePort: articlePort, onSettingsTapped: { showSettings = true })
            .sheet(isPresented: $showSettings) {
                SettingsPlaceholderView(authFeature: authFeature)
            }
    }
}

// 설정 placeholder (M6에서 확장)
struct SettingsPlaceholderView: View {
    let authFeature: AuthFeature
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            List {
                Button(role: .destructive) {
                    Task {
                        await authFeature.send(.signOut)
                        dismiss()
                    }
                } label: {
                    Label("로그아웃", systemImage: "rectangle.portrait.and.arrow.right")
                }
            }
            .navigationTitle("설정")
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("닫기") { dismiss() }
                }
            }
        }
    }
}
