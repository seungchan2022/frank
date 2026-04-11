import Foundation

/// Rust API 서버 POST /api/me/articles/like 어댑터.
///
/// MVP7 M2: 기사 좋아요 처리 — 키워드 추출 + 가중치 누적.
struct APILikesAdapter: LikesPort {
    private let auth: any AuthPort
    private let serverURL: URL
    private let session: URLSession

    init(auth: any AuthPort, serverConfig: ServerConfig, session: URLSession = .shared) {
        self.auth = auth
        self.serverURL = serverConfig.url
        self.session = session
    }

    func likeArticle(title: String, snippet: String?) async throws -> LikeResult {
        let token = try await auth.getAccessToken()

        guard let requestURL = URL(string: "/api/me/articles/like", relativeTo: serverURL) else {
            throw APILikesError.invalidURL
        }

        var request = URLRequest(url: requestURL)
        request.httpMethod = "POST"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let body = LikeArticleBody(title: title, snippet: snippet)
        request.httpBody = try jsonEncoder.encode(body)

        let (data, response) = try await session.data(for: request)

        guard let http = response as? HTTPURLResponse else {
            throw APILikesError.invalidResponse
        }

        guard (200...299).contains(http.statusCode) else {
            switch http.statusCode {
            case 401: throw APILikesError.unauthorized
            default: throw APILikesError.httpError(statusCode: http.statusCode)
            }
        }

        let dto = try jsonDecoder.decode(LikeArticleResponseDTO.self, from: data)
        return LikeResult(keywords: dto.keywords, totalLikes: dto.totalLikes)
    }
}

// MARK: - DTO

private struct LikeArticleBody: Encodable {
    let title: String
    let snippet: String?
}

private struct LikeArticleResponseDTO: Decodable {
    let keywords: [String]
    let totalLikes: Int

    enum CodingKeys: String, CodingKey {
        case keywords
        case totalLikes = "total_likes"
    }
}

// MARK: - JSON

private let jsonEncoder: JSONEncoder = {
    let encoder = JSONEncoder()
    return encoder
}()

private let jsonDecoder: JSONDecoder = {
    let decoder = JSONDecoder()
    return decoder
}()

// MARK: - Errors

enum APILikesError: LocalizedError, Equatable {
    case invalidURL
    case invalidResponse
    case unauthorized
    case httpError(statusCode: Int)

    var errorDescription: String? {
        switch self {
        case .invalidURL: "Invalid likes endpoint URL"
        case .invalidResponse: "Invalid response from server"
        case .unauthorized: "Unauthorized (401)"
        case .httpError(let code): "HTTP error: \(code)"
        }
    }
}
