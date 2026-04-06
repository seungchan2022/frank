import SwiftUI

struct SplashView: View {
    let feature: AuthFeature

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "newspaper.fill")
                .font(.system(size: 48))
                .foregroundStyle(.primary)
            Text("Frank")
                .font(.title)
                .fontWeight(.bold)
            ProgressView()
        }
        .task {
            await feature.send(.checkSession)
        }
    }
}
