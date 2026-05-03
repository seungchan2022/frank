import Foundation

/// Rust API 서버 POST /api/me/rewrite 호출 어댑터.
///
/// occupation 미설정 시 서버 400(occupation_required) → APIRewriteError.occupationRequired 에러 반환.
/// 요약과 동일하게 타임아웃 70초 설정.
struct APIRewriteAdapter: RewritePort {
    private let auth: any AuthPort
    private let serverURL: URL
    private let session: URLSession

    init(auth: any AuthPort, serverConfig: ServerConfig, session: URLSession = .shared) {
        self.auth = auth
        self.serverURL = serverConfig.url
        self.session = session
    }

    func rewrite(url: String, title: String) async throws -> RewriteResult {
        let token = try await auth.getAccessToken()

        guard let requestURL = URL(string: "/api/me/rewrite", relativeTo: serverURL) else {
            throw APIRewriteError.invalidURL
        }

        var request = URLRequest(url: requestURL)
        request.httpMethod = "POST"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.timeoutInterval = 70
        request.httpBody = try JSONEncoder().encode(RewriteRequestBody(url: url, title: title))

        let (data, response) = try await session.data(for: request)

        guard let http = response as? HTTPURLResponse else {
            throw APIRewriteError.invalidResponse
        }

        guard (200...299).contains(http.statusCode) else {
            switch http.statusCode {
            case 400: throw APIRewriteError.occupationRequired
            case 401: throw APIRewriteError.unauthorized
            case 422: throw APIRewriteError.crawlFailed
            case 503: throw APIRewriteError.llmUnavailable
            case 504: throw APIRewriteError.timeout
            default: throw APIRewriteError.httpError(statusCode: http.statusCode)
            }
        }

        return try JSONDecoder().decode(RewriteResult.self, from: data)
    }
}

// MARK: - Request Body

private struct RewriteRequestBody: Encodable {
    let url: String
    let title: String
}

// MARK: - Errors

enum APIRewriteError: LocalizedError, Equatable {
    case invalidURL
    case invalidResponse
    case occupationRequired
    case unauthorized
    case crawlFailed
    case llmUnavailable
    case timeout
    case httpError(statusCode: Int)

    var errorDescription: String? {
        switch self {
        case .invalidURL: "Invalid rewrite endpoint URL"
        case .invalidResponse: "Invalid response from server"
        case .occupationRequired: "직업을 먼저 설정해 주세요."
        case .unauthorized: "Unauthorized (401)"
        case .crawlFailed: "기사 내용을 불러오지 못했습니다."
        case .llmUnavailable: "재작성 서비스에 문제가 생겼습니다. 잠시 후 다시 시도해주세요."
        case .timeout: "재작성 시간이 초과됐습니다."
        case .httpError(let code): "HTTP error: \(code)"
        }
    }
}
