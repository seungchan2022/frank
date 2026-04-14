import Foundation

/// Rust API 서버를 호출하는 ArticlePort 구현체.
///
/// MVP5 M1: GET /api/me/feed — 검색 API 직접 호출 결과 반환 (DB 저장 없음)
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

    /// timestamptz microseconds 형식 파싱용 커스텀 디코더
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

    func fetchFeed(tagId: UUID?, noCache: Bool = false) async throws -> [FeedItem] {
        var components = URLComponents()
        components.path = "/api/me/feed"
        // MVP6 M3: tag_id 있으면 해당 태그만 서버에서 필터링
        if let tagId {
            components.queryItems = [URLQueryItem(name: "tag_id", value: tagId.uuidString)]
        }

        var request = try await makeRequest(components: components, method: "GET")
        // MVP10 M3: pull-to-refresh 시 서버 TTL 캐시 우회
        if noCache {
            request.setValue("no-cache", forHTTPHeaderField: "Cache-Control")
        }
        let dtos: [FeedItemDTO] = try await decode(request: request)
        return try dtos.map { try $0.toDomain() }
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

/// GET /me/feed 응답 DTO — ephemeral, id 없음
private struct FeedItemDTO: Decodable {
    let title: String
    let url: String
    let snippet: String?
    let source: String
    let publishedAt: Date?
    let tagId: UUID?
    /// MVP6 M1: 썸네일 이미지 URL 문자열 (없으면 nil)
    let imageUrl: String?

    enum CodingKeys: String, CodingKey {
        case title, url, source, snippet
        case publishedAt = "published_at"
        case tagId = "tag_id"
        case imageUrl = "image_url"
    }

    func toDomain() throws -> FeedItem {
        guard let parsedURL = URL(string: url) else {
            throw APIArticleError.invalidArticleURL(url)
        }
        // imageUrl 파싱 실패 시 nil (에러 아님)
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
