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
        case .checkingSession:
            SplashView(feature: feature)
        case .unauthenticated, .authenticating:
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
    let dependencies: AppDependencies
    @State private var feedFeature: FeedFeature
    @State private var settingsFeature: SettingsFeature?

    init(dependencies: AppDependencies, authFeature: AuthFeature) {
        self.authFeature = authFeature
        self.dependencies = dependencies
        self._feedFeature = State(initialValue: FeedFeature(
            article: dependencies.article,
            collect: dependencies.collect,
            tag: dependencies.tag
        ))
    }

    var body: some View {
        FeedView(feature: feedFeature, articlePort: dependencies.article, onSettingsTapped: {
            settingsFeature = SettingsFeature(tag: dependencies.tag, auth: dependencies.auth)
        })
        .sheet(item: $settingsFeature) { feature in
            SettingsView(feature: feature, authFeature: authFeature, onTagsSaved: {
                settingsFeature = nil
                Task { await feedFeature.send(.reloadAfterTagChange) }
            })
        }
    }
}
