import SwiftUI
import Supabase

@main
struct FrankApp: App {
    @State private var authFeature: AuthFeature

    init() {
        Log.app.notice("FrankApp launched")
        let dependencies = AppDependencies.live()
        _authFeature = State(initialValue: AuthFeature(auth: dependencies.auth))
    }

    var body: some Scene {
        WindowGroup {
            RootView(feature: authFeature)
        }
    }
}

struct RootView: View {
    let feature: AuthFeature

    var body: some View {
        switch feature.state {
        case .checkingSession, .authenticating:
            SplashView(feature: feature)
        case .unauthenticated:
            LoginView(feature: feature)
        case .authenticated(let profile):
            // TODO: M3 온보딩, M4 피드 분기 추가
            ContentPlaceholderView(profile: profile, feature: feature)
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
