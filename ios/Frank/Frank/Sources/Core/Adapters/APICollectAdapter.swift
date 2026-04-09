import Foundation

struct APICollectAdapter: CollectPort {
    private let auth: any AuthPort
    private let serverURL: URL

    init(auth: any AuthPort, serverConfig: ServerConfig) {
        self.auth = auth
        self.serverURL = serverConfig.url
    }

    func triggerCollect() async throws -> Int {
        try await postAndExtractCount(
            path: "/api/me/collect",
            key: "collected"
        )
    }

    // MARK: - Private

    private func postAndExtractCount(path: String, key: String, timeoutInterval: TimeInterval = 30) async throws -> Int {
        let token = try await auth.getAccessToken()

        guard let url = URL(string: path, relativeTo: serverURL) else {
            throw APICollectError.invalidURL(path)
        }

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.timeoutInterval = timeoutInterval

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse else {
            throw APICollectError.invalidResponse
        }

        guard (200...299).contains(httpResponse.statusCode) else {
            throw APICollectError.httpError(statusCode: httpResponse.statusCode)
        }

        let decoded = try JSONDecoder().decode([String: Int].self, from: data)

        guard let count = decoded[key] else {
            throw APICollectError.missingKey(key)
        }

        return count
    }
}

// MARK: - Errors

enum APICollectError: LocalizedError {
    case invalidURL(String)
    case invalidResponse
    case httpError(statusCode: Int)
    case missingKey(String)

    var errorDescription: String? {
        switch self {
        case .invalidURL(let path):
            "Invalid URL for path: \(path)"
        case .invalidResponse:
            "Invalid response from server"
        case .httpError(let statusCode):
            "HTTP error: \(statusCode)"
        case .missingKey(let key):
            "Missing key in response: \(key)"
        }
    }
}
