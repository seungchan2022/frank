import Foundation

/// Rust API 서버 GET /api/me/articles/related 어댑터.
///
/// MVP7 M3: 연관 기사 조회 — 현재 기사의 title/snippet 기반 유사 기사 반환.
struct APIRelatedAdapter: RelatedPort {
    private let auth: any AuthPort
    private let serverURL: URL
    private let session: URLSession
    private let decoder: JSONDecoder

    init(auth: any AuthPort, serverConfig: ServerConfig, session: URLSession = .shared) {
        self.auth = auth
        self.serverURL = serverConfig.url
        self.session = session
        self.decoder = Self.makeDecoder()
    }

    func fetchRelated(title: String, snippet: String?) async throws -> [FeedItem] {
        let token = try await auth.getAccessToken()

        var components = URLComponents()
        components.path = "/api/me/articles/related"
        var queryItems = [URLQueryItem(name: "title", value: title)]
        if let snippet {
            queryItems.append(URLQueryItem(name: "snippet", value: snippet))
        }
        components.queryItems = queryItems

        guard let url = components.url(relativeTo: serverURL) else {
            throw APIRelatedError.invalidURL
        }

        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")

        let (data, response) = try await session.data(for: request)

        guard let http = response as? HTTPURLResponse else {
            throw APIRelatedError.invalidResponse
        }

        guard (200...299).contains(http.statusCode) else {
            switch http.statusCode {
            case 401: throw APIRelatedError.unauthorized
            default: throw APIRelatedError.httpError(statusCode: http.statusCode)
            }
        }

        let dtos = try decoder.decode([RelatedFeedItemDTO].self, from: data)
        return try dtos.map { try $0.toDomain() }
    }

    // MARK: - Private

    private static func makeDecoder() -> JSONDecoder {
        let decoder = JSONDecoder()
        let isoFractional = ISO8601DateFormatter()
        isoFractional.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        let isoBasic = ISO8601DateFormatter()
        isoBasic.formatOptions = [.withInternetDateTime]

        decoder.dateDecodingStrategy = .custom { decoder in
            let container = try decoder.singleValueContainer()
            let str = try container.decode(String.self)
            if let date = isoBasic.date(from: str) { return date }
            if let date = isoFractional.date(from: str) { return date }
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Invalid ISO8601 date: \(str)"
            )
        }
        return decoder
    }
}

// MARK: - DTO

private struct RelatedFeedItemDTO: Decodable {
    let title: String
    let url: String
    let snippet: String?
    let source: String
    let publishedAt: Date?
    let tagId: UUID?
    let imageUrl: String?

    enum CodingKeys: String, CodingKey {
        case title, url, source, snippet
        case publishedAt = "published_at"
        case tagId = "tag_id"
        case imageUrl = "image_url"
    }

    func toDomain() throws -> FeedItem {
        guard let parsedURL = URL(string: url) else {
            throw APIRelatedError.invalidArticleURL(url)
        }
        let parsedImageUrl = imageUrl.flatMap { URL(string: $0) }
        return FeedItem(
            title: title,
            url: parsedURL,
            source: source,
            publishedAt: publishedAt,
            tagId: tagId,
            snippet: snippet,
            imageUrl: parsedImageUrl
        )
    }
}

// MARK: - Errors

enum APIRelatedError: LocalizedError, Equatable {
    case invalidURL
    case invalidArticleURL(String)
    case invalidResponse
    case unauthorized
    case httpError(statusCode: Int)

    var errorDescription: String? {
        switch self {
        case .invalidURL: "Invalid related articles endpoint URL"
        case .invalidArticleURL(let raw): "Invalid article URL in response: \(raw)"
        case .invalidResponse: "Invalid response from server"
        case .unauthorized: "Unauthorized (401)"
        case .httpError(let code): "HTTP error: \(code)"
        }
    }
}
