import SwiftUI

@main
struct FrankApp: App {
    init() {
        Log.app.notice("FrankApp launched")
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

struct ContentView: View {
    var body: some View {
        Text("Frank")
    }
}
