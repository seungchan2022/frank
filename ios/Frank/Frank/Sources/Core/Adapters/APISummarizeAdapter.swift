import Foundation

/// Rust API 서버 POST /api/me/summarize 호출 어댑터.
///
/// 요약 요청은 서버 측에서 크롤링 + LLM 처리가 포함되어 오래 걸릴 수 있어,
/// 타임아웃을 70초로 설정한다 (서버 60초 + 10초 여유).
/// 피드/즐겨찾기 등 일반 요청과 분리해 URLSession을 별도 주입할 수 있다.
struct APISummarizeAdapter: SummarizePort {
    private let auth: any AuthPort
    private let serverURL: URL
    private let session: URLSession

    init(auth: any AuthPort, serverConfig: ServerConfig, session: URLSession = .shared) {
        self.auth = auth
        self.serverURL = serverConfig.url
        self.session = session
    }

    func summarize(url: String, title: String) async throws -> SummaryResult {
        let token = try await auth.getAccessToken()

        guard let requestURL = URL(string: "/api/me/summarize", relativeTo: serverURL) else {
            throw APISummarizeError.invalidURL
        }

        var request = URLRequest(url: requestURL)
        request.httpMethod = "POST"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        // MVP7 M1: 요약 요청 타임아웃 70초 (서버 60초 + 10초 여유 — 서버 정상 에러 응답 보장)
        request.timeoutInterval = 70
        request.httpBody = try JSONEncoder().encode(SummarizeRequestBody(url: url, title: title))

        let (data, response) = try await session.data(for: request)

        guard let http = response as? HTTPURLResponse else {
            throw APISummarizeError.invalidResponse
        }

        guard (200...299).contains(http.statusCode) else {
            switch http.statusCode {
            case 400: throw APISummarizeError.badRequest
            case 401: throw APISummarizeError.unauthorized
            case 504: throw APISummarizeError.timeout
            default: throw APISummarizeError.httpError(statusCode: http.statusCode)
            }
        }

        return try JSONDecoder().decode(SummaryResult.self, from: data)
    }
}

// MARK: - Request Body

private struct SummarizeRequestBody: Encodable {
    let url: String
    let title: String
}

// MARK: - Errors

enum APISummarizeError: LocalizedError, Equatable {
    case invalidURL
    case invalidResponse
    case badRequest
    case unauthorized
    case timeout
    case httpError(statusCode: Int)

    var errorDescription: String? {
        switch self {
        case .invalidURL: "Invalid summarize endpoint URL"
        case .invalidResponse: "Invalid response from server"
        case .badRequest: "잘못된 요청입니다."
        case .unauthorized: "Unauthorized (401)"
        case .timeout: "요약 요청이 시간을 초과했습니다. 다시 시도해주세요."
        case .httpError(let code): "HTTP error: \(code)"
        }
    }
}
