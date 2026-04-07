import Foundation

/// Rust API 서버를 호출하는 TagPort 구현체.
///
/// - `GET /api/tags` — 전체 태그
/// - `GET /api/me/tags` — 내가 선택한 태그 ID 배열
/// - `POST /api/me/tags` — 내 태그 전체 교체 (`{tag_ids: [...]}`)
struct APITagAdapter: TagPort {
    private let auth: any AuthPort
    private let serverURL: URL
    private let session: URLSession

    init(auth: any AuthPort, serverConfig: ServerConfig, session: URLSession = .shared) {
        self.auth = auth
        self.serverURL = serverConfig.url
        self.session = session
    }

    func fetchAllTags() async throws -> [Tag] {
        let request = try await makeRequest(path: "/api/tags", method: "GET")
        let dtos: [TagDTO] = try await decode(request: request)
        return dtos.map { $0.toDomain() }
    }

    func fetchMyTagIds() async throws -> [UUID] {
        let request = try await makeRequest(path: "/api/me/tags", method: "GET")
        let strings: [String] = try await decode(request: request)
        // 잘못된 UUID를 silent drop 하지 않고 명시적 throw —
        // 부분 데이터 손실을 사용자에게 숨기는 silent failure 방지.
        return try strings.map { raw in
            guard let uuid = UUID(uuidString: raw) else {
                throw APITagError.invalidUUIDInResponse(raw)
            }
            return uuid
        }
    }

    func saveMyTags(tagIds: [UUID]) async throws {
        var request = try await makeRequest(path: "/api/me/tags", method: "POST")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        let body = SaveTagsRequest(tagIds: tagIds.map(\.uuidString))
        request.httpBody = try JSONEncoder().encode(body)
        try await sendIgnoringBody(request: request)
    }

    // MARK: - Private

    private func makeRequest(path: String, method: String) async throws -> URLRequest {
        let token = try await auth.getAccessToken()
        guard let url = URL(string: path, relativeTo: serverURL) else {
            throw APITagError.invalidURL(path)
        }
        var request = URLRequest(url: url)
        request.httpMethod = method
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        return request
    }

    private func decode<T: Decodable>(request: URLRequest) async throws -> T {
        let data = try await sendValidating(request: request)
        return try JSONDecoder().decode(T.self, from: data)
    }

    private func sendValidating(request: URLRequest) async throws -> Data {
        let (data, response) = try await session.data(for: request)
        try validate(response: response)
        return data
    }

    private func sendIgnoringBody(request: URLRequest) async throws {
        let (_, response) = try await session.data(for: request)
        try validate(response: response)
    }

    private func validate(response: URLResponse) throws {
        guard let http = response as? HTTPURLResponse else {
            throw APITagError.invalidResponse
        }
        guard (200...299).contains(http.statusCode) else {
            if http.statusCode == 401 {
                throw APITagError.unauthorized
            }
            throw APITagError.httpError(statusCode: http.statusCode)
        }
    }
}

// MARK: - DTOs

private struct TagDTO: Decodable {
    let id: UUID
    let name: String
    let category: String?

    func toDomain() -> Tag {
        Tag(id: id, name: name, category: category)
    }
}

private struct SaveTagsRequest: Encodable {
    let tagIds: [String]

    enum CodingKeys: String, CodingKey {
        case tagIds = "tag_ids"
    }
}

// MARK: - Errors

enum APITagError: LocalizedError, Equatable {
    case invalidURL(String)
    case invalidUUIDInResponse(String)
    case invalidResponse
    case unauthorized
    case httpError(statusCode: Int)

    var errorDescription: String? {
        switch self {
        case .invalidURL(let path):
            "Invalid URL for path: \(path)"
        case .invalidUUIDInResponse(let raw):
            "Invalid UUID in response: \(raw)"
        case .invalidResponse:
            "Invalid response from server"
        case .unauthorized:
            "Unauthorized (401)"
        case .httpError(let code):
            "HTTP error: \(code)"
        }
    }
}
