import Foundation

/// Rust API 서버 POST /api/me/favorites/quiz 어댑터.
///
/// MVP7 M4: 즐겨찾기 기사로 퀴즈 생성.
struct APIQuizAdapter: QuizPort {
    private let auth: any AuthPort
    private let serverURL: URL
    private let session: URLSession
    private let decoder: JSONDecoder

    init(auth: any AuthPort, serverConfig: ServerConfig, session: URLSession = .shared) {
        self.auth = auth
        self.serverURL = serverConfig.url
        self.session = session
        self.decoder = JSONDecoder()
    }

    func generateQuiz(url: String) async throws -> [QuizQuestion] {
        let token = try await auth.getAccessToken()

        guard let endpoint = URL(string: "/api/me/favorites/quiz", relativeTo: serverURL) else {
            throw APIQuizError.invalidURL
        }

        var request = URLRequest(url: endpoint)
        request.httpMethod = "POST"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try JSONEncoder().encode(QuizRequestBody(url: url))

        let (data, response) = try await session.data(for: request)

        guard let http = response as? HTTPURLResponse else {
            throw APIQuizError.invalidResponse
        }

        switch http.statusCode {
        case 200:
            let dto = try decoder.decode(QuizResponseDTO.self, from: data)
            return dto.questions
        case 401:
            throw APIQuizError.unauthorized
        case 404:
            throw APIQuizError.notFound
        case 503:
            throw APIQuizError.serviceUnavailable
        default:
            throw APIQuizError.httpError(statusCode: http.statusCode)
        }
    }
}

// MARK: - DTO

private struct QuizRequestBody: Encodable {
    let url: String
}

private struct QuizResponseDTO: Decodable {
    let questions: [QuizQuestion]
}

// MARK: - Errors

enum APIQuizError: LocalizedError, Equatable {
    case invalidURL
    case invalidResponse
    case unauthorized
    case notFound
    case serviceUnavailable
    case httpError(statusCode: Int)

    var errorDescription: String? {
        switch self {
        case .invalidURL: "Invalid quiz endpoint URL"
        case .invalidResponse: "Invalid response from server"
        case .unauthorized: "Unauthorized (401)"
        case .notFound: "즐겨찾기에 없는 기사입니다. (404)"
        case .serviceUnavailable: "퀴즈 생성에 실패했습니다. 잠시 후 다시 시도해주세요. (503)"
        case .httpError(let code): "HTTP error: \(code)"
        }
    }
}
