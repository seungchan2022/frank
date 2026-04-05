import SwiftUI

enum Route: Hashable {
    case feed
    case articleDetail(id: UUID)
    case settings
}

@Observable
@MainActor
final class Router {
    var path = NavigationPath()

    func navigate(to route: Route) {
        Log.router.info("navigate → \(String(describing: route))")
        path.append(route)
    }

    func pop() {
        guard !path.isEmpty else { return }
        Log.router.info("pop (depth: \(self.path.count))")
        path.removeLast()
    }

    func popToRoot() {
        Log.router.info("popToRoot")
        path = NavigationPath()
    }
}
