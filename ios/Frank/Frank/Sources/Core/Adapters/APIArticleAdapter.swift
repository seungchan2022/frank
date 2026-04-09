import Foundation

/// Rust API 서버를 호출하는 ArticlePort 구현체.
///
/// - `GET /api/me/articles?limit=&offset=&tag_id=` — 본인 기사 목록
/// - `GET /api/me/articles/:id` — 단건 (본인 기사만)
struct APIArticleAdapter: ArticlePort {
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

    /// PostgREST가 반환하는 timestamptz는 microseconds 6자리(`.350714Z`) 형식이다.
    /// `JSONDecoder.dateDecodingStrategy = .iso8601`는 fractional seconds를 미지원,
    /// `.withFractionalSeconds`도 millisecond 3자리까지만 처리하므로 모두 실패한다.
    /// → microseconds → milliseconds로 truncate 후 ISO8601 파싱하는 custom strategy.
    private static func makeDecoder() -> JSONDecoder {
        let decoder = JSONDecoder()
        let isoFractional = ISO8601DateFormatter()
        isoFractional.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        let isoBasic = ISO8601DateFormatter()
        isoBasic.formatOptions = [.withInternetDateTime]

        decoder.dateDecodingStrategy = .custom { decoder in
            let container = try decoder.singleValueContainer()
            let str = try container.decode(String.self)

            // 1) 표준 ISO8601 (zone 있음)
            if let date = isoBasic.date(from: str) { return date }

            // 2) fractional seconds 3자리 이내
            if let date = isoFractional.date(from: str) { return date }

            // 3) microseconds(6자리) 등 → 첫 3자리만 남기고 truncate
            if let truncated = Self.truncateFractionalSeconds(str),
               let date = isoFractional.date(from: truncated) {
                return date
            }

            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Invalid ISO8601 date: \(str)"
            )
        }
        return decoder
    }

    /// `2026-04-07T07:32:37.350714Z` → `2026-04-07T07:32:37.350Z`
    /// fractional seconds가 3자리를 초과하면 millisecond까지만 남기고 truncate.
    /// nil 반환 = truncate 대상 아님 (호출자가 다른 형식으로 시도).
    private static func truncateFractionalSeconds(_ str: String) -> String? {
        guard let dotIdx = str.firstIndex(of: ".") else { return nil }
        let afterDot = str.index(after: dotIdx)
        guard let zoneIdx = str[afterDot...].firstIndex(where: { $0 == "Z" || $0 == "+" || $0 == "-" }) else {
            return nil
        }
        let fracLen = str.distance(from: afterDot, to: zoneIdx)
        guard fracLen > 3 else { return nil }
        let truncEnd = str.index(afterDot, offsetBy: 3)
        return String(str[..<truncEnd]) + String(str[zoneIdx...])
    }

    func fetchArticles(filter: ArticleFilter) async throws -> [Article] {
        var components = URLComponents()
        components.path = "/api/me/articles"
        var items: [URLQueryItem] = [
            URLQueryItem(name: "limit", value: String(filter.limit)),
            URLQueryItem(name: "offset", value: String(filter.offset))
        ]
        if let tagId = filter.tagId {
            items.append(URLQueryItem(name: "tag_id", value: tagId.uuidString))
        }
        components.queryItems = items

        let request = try await makeRequest(components: components, method: "GET")
        let dtos: [ArticleDTO] = try await decode(request: request)
        return try dtos.map { try $0.toDomain() }
    }

    func fetchArticle(id: UUID) async throws -> Article {
        var components = URLComponents()
        components.path = "/api/me/articles/\(id.uuidString)"

        let request = try await makeRequest(components: components, method: "GET")
        let dto: ArticleDTO = try await decode(request: request)
        return try dto.toDomain()
    }

    // MARK: - Private

    private func makeRequest(components: URLComponents, method: String) async throws -> URLRequest {
        let token = try await auth.getAccessToken()
        guard let url = components.url(relativeTo: serverURL) else {
            throw APIArticleError.invalidURL(components.path)
        }
        var request = URLRequest(url: url)
        request.httpMethod = method
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        return request
    }

    private func decode<T: Decodable>(request: URLRequest) async throws -> T {
        let (data, response) = try await session.data(for: request)
        try validate(response: response)
        return try decoder.decode(T.self, from: data)
    }

    private func validate(response: URLResponse) throws {
        guard let http = response as? HTTPURLResponse else {
            throw APIArticleError.invalidResponse
        }
        guard (200...299).contains(http.statusCode) else {
            switch http.statusCode {
            case 401: throw APIArticleError.unauthorized
            case 404: throw APIArticleError.notFound
            default: throw APIArticleError.httpError(statusCode: http.statusCode)
            }
        }
    }
}

// MARK: - DTO

private struct ArticleDTO: Decodable {
    let id: UUID
    let userId: UUID
    let tagId: UUID?
    let title: String
    let url: String
    let snippet: String?
    let source: String
    let publishedAt: Date?
    let createdAt: Date?

    enum CodingKeys: String, CodingKey {
        case id, title, url, source, snippet
        case userId = "user_id"
        case tagId = "tag_id"
        case publishedAt = "published_at"
        case createdAt = "created_at"
    }

    /// 잘못된 URL을 silent fallback 하지 않고 명시적 throw — 백엔드 데이터 오염을
    /// 사용자에게 잘못된 링크로 노출하는 것을 차단한다 (보안 + 데이터 무결성).
    func toDomain() throws -> Article {
        guard let parsedURL = URL(string: url) else {
            throw APIArticleError.invalidArticleURL(url)
        }
        return Article(
            id: id,
            userId: userId,
            title: title,
            url: parsedURL,
            source: source,
            publishedAt: publishedAt,
            tagId: tagId,
            snippet: snippet,
            createdAt: createdAt
        )
    }
}

// MARK: - Errors

enum APIArticleError: LocalizedError, Equatable {
    case invalidURL(String)
    case invalidArticleURL(String)
    case invalidResponse
    case unauthorized
    case notFound
    case httpError(statusCode: Int)

    var errorDescription: String? {
        switch self {
        case .invalidURL(let path):
            "Invalid URL for path: \(path)"
        case .invalidArticleURL(let raw):
            "Invalid article URL in response: \(raw)"
        case .invalidResponse:
            "Invalid response from server"
        case .unauthorized:
            "Unauthorized (401)"
        case .notFound:
            "Not found (404)"
        case .httpError(let code):
            "HTTP error: \(code)"
        }
    }
}
