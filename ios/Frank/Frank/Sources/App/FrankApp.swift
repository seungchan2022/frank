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
                MainTabView(
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

/// MVP5 M3: MainTabView — 피드 탭 + 스크랩 탭.
/// FavoritesFeature는 여기서 1개 생성 → 양쪽 탭에 공유.
struct MainTabView: View {
    let authFeature: AuthFeature
    let dependencies: AppDependencies
    @State private var favoritesFeature: FavoritesFeature

    init(dependencies: AppDependencies, authFeature: AuthFeature) {
        self.authFeature = authFeature
        self.dependencies = dependencies
        self._favoritesFeature = State(initialValue: FavoritesFeature(favorites: dependencies.favorites))
    }

    var body: some View {
        TabView {
            FeedContainerView(
                dependencies: dependencies,
                authFeature: authFeature,
                favoritesFeature: favoritesFeature
            )
            .tabItem {
                Label("피드", systemImage: "newspaper.fill")
            }

            FavoritesContainerView(
                feature: favoritesFeature,
                summarize: dependencies.summarize
            )
            .tabItem {
                Label("스크랩", systemImage: "bookmark.fill")
            }
        }
    }
}

struct FeedContainerView: View {
    let authFeature: AuthFeature
    let dependencies: AppDependencies
    let favoritesFeature: FavoritesFeature
    @State private var feedFeature: FeedFeature
    @State private var settingsFeature: SettingsFeature?

    init(dependencies: AppDependencies, authFeature: AuthFeature, favoritesFeature: FavoritesFeature) {
        self.authFeature = authFeature
        self.dependencies = dependencies
        self.favoritesFeature = favoritesFeature
        self._feedFeature = State(initialValue: FeedFeature(
            article: dependencies.article,
            tag: dependencies.tag
        ))
    }

    var body: some View {
        FeedView(
            feature: feedFeature,
            summarize: dependencies.summarize,
            favoritesFeature: favoritesFeature,
            onSettingsTapped: {
                settingsFeature = SettingsFeature(tag: dependencies.tag, auth: dependencies.auth)
            }
        )
        .sheet(item: $settingsFeature) { feature in
            SettingsView(feature: feature, authFeature: authFeature, onTagsSaved: {
                settingsFeature = nil
                Task { await feedFeature.send(.reloadAfterTagChange) }
            })
        }
    }
}

struct FavoritesContainerView: View {
    let feature: FavoritesFeature
    let summarize: any SummarizePort

    var body: some View {
        FavoritesView(feature: feature, summarize: summarize)
    }
}
