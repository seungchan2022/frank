import Foundation

struct ServerConfig: Sendable {
    let url: URL

    static var live: ServerConfig {
        // 1) Info.plist에서 읽기 시도
        if let urlString = Bundle.main.infoDictionary?["SERVER_URL"] as? String,
           !urlString.isEmpty,
           let url = URL(string: urlString) {
            return ServerConfig(url: url)
        }

        // 2) Secrets.plist 파일에서 읽기 시도
        if let path = Bundle.main.path(forResource: "Secrets", ofType: "plist"),
           let dict = NSDictionary(contentsOfFile: path),
           let urlString = dict["SERVER_URL"] as? String,
           let url = URL(string: urlString) {
            return ServerConfig(url: url)
        }

        Log.app.warning("Server config not found — using localhost fallback")
        guard let fallbackURL = URL(string: "http://localhost:8080") else {
            fatalError("Invalid fallback URL literal — this is a programmer error")
        }
        return ServerConfig(url: fallbackURL)
    }
}
