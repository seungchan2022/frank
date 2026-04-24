import Foundation

// MARK: - Error

enum ServerConfigError: Error, Equatable {
    case invalidURL(String)
    case missing
}

// MARK: - ServerConfig

struct ServerConfig: Sendable {
    let url: URL

    /// URL 문자열을 파싱해 ServerConfig를 생성한다.
    /// 빈 문자열이거나 URL 파싱에 실패하면 `ServerConfigError.invalidURL`을 던진다.
    static func make(urlString: String) throws(ServerConfigError) -> ServerConfig {
        guard !urlString.isEmpty, let url = URL(string: urlString) else {
            throw ServerConfigError.invalidURL(urlString)
        }
        return ServerConfig(url: url)
    }

    /// 빌드 환경에 따라 적절한 ServerConfig를 반환한다.
    ///
    /// - 시뮬레이터: 항상 `http://localhost:8080` (컴파일 타임 분기)
    /// - 실기기: Info.plist → Secrets.plist 순서로 탐색, 모두 없으면 에러
    static func live() throws(ServerConfigError) -> ServerConfig {
        #if targetEnvironment(simulator)
        return try make(urlString: "http://localhost:8080")
        #else
        // 1) Info.plist에서 읽기 시도
        if let urlString = Bundle.main.infoDictionary?["SERVER_URL"] as? String,
           !urlString.isEmpty {
            return try make(urlString: urlString)
        }

        // 2) Secrets.plist 파일에서 읽기 시도
        if let path = Bundle.main.path(forResource: "Secrets", ofType: "plist"),
           let dict = NSDictionary(contentsOfFile: path),
           let urlString = dict["SERVER_URL"] as? String,
           !urlString.isEmpty {
            return try make(urlString: urlString)
        }

        // 3) 실기기에서 URL을 찾지 못하면 에러 — 폴백 URL 없음
        Log.app.error("ServerConfig: SERVER_URL not found in Info.plist or Secrets.plist")
        throw ServerConfigError.missing
        #endif
    }
}
