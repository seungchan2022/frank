import SwiftUI
import Supabase

@main
struct FrankApp: App {
    @State private var bootstrap: AppBootstrap

    init() {
        Log.app.notice("FrankApp launched")
        _bootstrap = State(initialValue: AppDependencies.bootstrap())
    }

    var body: some Scene {
        WindowGroup {
            switch bootstrap {
            case .ready(let deps):
                RootView(
                    feature: AuthFeature(auth: deps.auth),
                    dependencies: deps
                )
            case .configError(let error):
                ConfigErrorView(error: error)
            }
        }
    }
}

// MARK: - RootView

struct RootView: View {
    @State var feature: AuthFeature
    let dependencies: AppDependencies

    init(feature: AuthFeature, dependencies: AppDependencies) {
        _feature = State(initialValue: feature)
        self.dependencies = dependencies
    }

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

// MARK: - ConfigErrorView

/// 서버 URL 설정 오류 시 표시하는 전용 에러 화면.
/// API 어댑터가 생성되지 않으므로 빈 화면이나 크래시 없이 안내 메시지를 유지한다.
struct ConfigErrorView: View {
    let error: ServerConfigError

    private var message: String {
        switch error {
        case .missing:
            return "서버 URL이 설정되어 있지 않습니다.\nInfo.plist 또는 Secrets.plist에 SERVER_URL을 추가해주세요."
        case .invalidURL(let raw):
            return "서버 URL이 올바르지 않습니다: \(raw)\nInfo.plist 또는 Secrets.plist에서 SERVER_URL을 확인해주세요."
        }
    }

    var body: some View {
        VStack(spacing: 24) {
            Image(systemName: "exclamationmark.triangle.fill")
                .font(.system(size: 56))
                .foregroundStyle(.yellow)

            Text("설정 오류")
                .font(.title2.bold())

            Text(message)
                .font(.body)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 32)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

// MARK: - OnboardingContainerView

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

// MARK: - MainTabView

/// MVP5 M3: MainTabView — 피드 탭 + 스크랩 탭.
/// FavoritesFeature + LikesFeature는 여기서 1개씩 생성 → 양쪽 탭에 공유.
struct MainTabView: View {
    let authFeature: AuthFeature
    let dependencies: AppDependencies
    @State private var favoritesFeature: FavoritesFeature
    /// MVP7 M2: LikesFeature 루트 소유 — FeedView + ArticleDetailView 공유
    @State private var likesFeature: LikesFeature

    init(dependencies: AppDependencies, authFeature: AuthFeature) {
        self.authFeature = authFeature
        self.dependencies = dependencies
        self._favoritesFeature = State(initialValue: FavoritesFeature(favorites: dependencies.favorites))
        self._likesFeature = State(initialValue: LikesFeature(likes: dependencies.likes))
    }

    var body: some View {
        TabView {
            FeedContainerView(
                dependencies: dependencies,
                authFeature: authFeature,
                favoritesFeature: favoritesFeature,
                likesFeature: likesFeature
            )
            .tabItem {
                Label("피드", systemImage: "newspaper.fill")
            }

            FavoritesContainerView(
                feature: favoritesFeature,
                summarize: dependencies.summarize,
                likesFeature: likesFeature,
                quiz: dependencies.quiz,
                wrongAnswer: dependencies.wrongAnswer,
                favorites: dependencies.favorites
            )
            .tabItem {
                Label("스크랩", systemImage: "bookmark.fill")
            }
        }
    }
}

// MARK: - FeedContainerView

struct FeedContainerView: View {
    let authFeature: AuthFeature
    let dependencies: AppDependencies
    let favoritesFeature: FavoritesFeature
    let likesFeature: LikesFeature
    @State private var feedFeature: FeedFeature
    @State private var settingsFeature: SettingsFeature?

    init(
        dependencies: AppDependencies,
        authFeature: AuthFeature,
        favoritesFeature: FavoritesFeature,
        likesFeature: LikesFeature
    ) {
        self.authFeature = authFeature
        self.dependencies = dependencies
        self.favoritesFeature = favoritesFeature
        self.likesFeature = likesFeature
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
            likesFeature: likesFeature,
            quiz: dependencies.quiz,
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

// MARK: - FavoritesContainerView

struct FavoritesContainerView: View {
    let feature: FavoritesFeature
    let summarize: any SummarizePort
    let likesFeature: LikesFeature
    let quiz: any QuizPort
    let wrongAnswer: any WrongAnswerPort
    let favorites: any FavoritesPort

    var body: some View {
        FavoritesView(
            feature: feature,
            summarize: summarize,
            likesFeature: likesFeature,
            quiz: quiz,
            wrongAnswer: wrongAnswer,
            favorites: favorites
        )
    }
}
