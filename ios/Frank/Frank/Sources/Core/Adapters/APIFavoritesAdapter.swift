import Foundation

/// Rust API 서버 /api/me/favorites CRUD 어댑터.
struct APIFavoritesAdapter: FavoritesPort {
    private let auth: any AuthPort
    private let serverURL: URL

    init(auth: any AuthPort, serverConfig: ServerConfig) {
        self.auth = auth
        self.serverURL = serverConfig.url
    }

    func addFavorite(item: FeedItem, summary: String?, insight: String?) async throws -> FavoriteItem {
        let token = try await auth.getAccessToken()

        guard let requestURL = URL(string: "/api/me/favorites", relativeTo: serverURL) else {
            throw APIFavoritesError.invalidURL
        }

        var request = URLRequest(url: requestURL)
        request.httpMethod = "POST"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let body = AddFavoriteBody(
            title: item.title,
            url: item.url.absoluteString,
            snippet: item.snippet,
            source: item.source,
            publishedAt: item.publishedAt,
            tagId: item.tagId,
            summary: summary,
            insight: insight
        )
        request.httpBody = try jsonEncoder.encode(body)

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let http = response as? HTTPURLResponse else {
            throw APIFavoritesError.invalidResponse
        }

        switch http.statusCode {
        case 200...299:
            return try jsonDecoder.decode(FavoriteItem.self, from: data)
        case 409:
            throw APIFavoritesError.conflict
        case 401:
            throw APIFavoritesError.unauthorized
        default:
            throw APIFavoritesError.httpError(statusCode: http.statusCode)
        }
    }

    func deleteFavorite(url: String) async throws {
        let token = try await auth.getAccessToken()

        guard let encodedURL = url.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed),
              let requestURL = URL(string: "/api/me/favorites?url=\(encodedURL)", relativeTo: serverURL) else {
            throw APIFavoritesError.invalidURL
        }

        var request = URLRequest(url: requestURL)
        request.httpMethod = "DELETE"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")

        let (_, response) = try await URLSession.shared.data(for: request)

        guard let http = response as? HTTPURLResponse else {
            throw APIFavoritesError.invalidResponse
        }

        guard (200...299).contains(http.statusCode) else {
            throw APIFavoritesError.httpError(statusCode: http.statusCode)
        }
    }

    func listFavorites() async throws -> [FavoriteItem] {
        let token = try await auth.getAccessToken()

        guard let requestURL = URL(string: "/api/me/favorites", relativeTo: serverURL) else {
            throw APIFavoritesError.invalidURL
        }

        var request = URLRequest(url: requestURL)
        request.httpMethod = "GET"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let http = response as? HTTPURLResponse else {
            throw APIFavoritesError.invalidResponse
        }

        guard (200...299).contains(http.statusCode) else {
            if http.statusCode == 401 { throw APIFavoritesError.unauthorized }
            throw APIFavoritesError.httpError(statusCode: http.statusCode)
        }

        return try jsonDecoder.decode([FavoriteItem].self, from: data)
    }
}

// MARK: - Request Body

private struct AddFavoriteBody: Encodable {
    let title: String
    let url: String
    let snippet: String?
    let source: String
    let publishedAt: Date?
    let tagId: UUID?
    let summary: String?
    let insight: String?

    enum CodingKeys: String, CodingKey {
        case title, url, snippet, source, summary, insight
        case publishedAt = "published_at"
        case tagId = "tag_id"
    }
}

// MARK: - JSON Encoder/Decoder

private let jsonEncoder: JSONEncoder = {
    let encoder = JSONEncoder()
    encoder.dateEncodingStrategy = .iso8601
    return encoder
}()

private let jsonDecoder: JSONDecoder = {
    let decoder = JSONDecoder()
    decoder.dateDecodingStrategy = .iso8601
    return decoder
}()

// MARK: - Errors

enum APIFavoritesError: LocalizedError, Equatable {
    case invalidURL
    case invalidResponse
    case unauthorized
    case conflict
    case httpError(statusCode: Int)

    var errorDescription: String? {
        switch self {
        case .invalidURL: "Invalid favorites endpoint URL"
        case .invalidResponse: "Invalid response from server"
        case .unauthorized: "Unauthorized (401)"
        case .conflict: "이미 즐겨찾기에 추가된 기사입니다."
        case .httpError(let code): "HTTP error: \(code)"
        }
    }
}
