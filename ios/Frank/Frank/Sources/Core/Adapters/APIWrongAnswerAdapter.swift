import Foundation

/// Rust API /api/me/quiz/wrong-answers 어댑터.
struct APIWrongAnswerAdapter: WrongAnswerPort {
    private let auth: any AuthPort
    private let serverURL: URL

    init(auth: any AuthPort, serverConfig: ServerConfig) {
        self.auth = auth
        self.serverURL = serverConfig.url
    }

    func save(params: SaveWrongAnswerParams) async throws -> WrongAnswer {
        let token = try await auth.getAccessToken()

        guard let endpoint = URL(string: "/api/me/quiz/wrong-answers", relativeTo: serverURL) else {
            throw APIWrongAnswerError.invalidURL
        }

        var request = URLRequest(url: endpoint)
        request.httpMethod = "POST"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try jsonEncoder.encode(params)

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let http = response as? HTTPURLResponse else {
            throw APIWrongAnswerError.invalidResponse
        }

        guard (200...299).contains(http.statusCode) else {
            if http.statusCode == 401 { throw APIWrongAnswerError.unauthorized }
            throw APIWrongAnswerError.httpError(statusCode: http.statusCode)
        }

        return try jsonDecoder.decode(WrongAnswer.self, from: data)
    }

    func list() async throws -> [WrongAnswer] {
        let token = try await auth.getAccessToken()

        guard let endpoint = URL(string: "/api/me/quiz/wrong-answers", relativeTo: serverURL) else {
            throw APIWrongAnswerError.invalidURL
        }

        var request = URLRequest(url: endpoint)
        request.httpMethod = "GET"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let http = response as? HTTPURLResponse else {
            throw APIWrongAnswerError.invalidResponse
        }

        guard (200...299).contains(http.statusCode) else {
            if http.statusCode == 401 { throw APIWrongAnswerError.unauthorized }
            throw APIWrongAnswerError.httpError(statusCode: http.statusCode)
        }

        return try jsonDecoder.decode([WrongAnswer].self, from: data)
    }

    func delete(id: UUID) async throws {
        let token = try await auth.getAccessToken()

        guard let endpoint = URL(
            string: "/api/me/quiz/wrong-answers/\(id.uuidString.lowercased())",
            relativeTo: serverURL
        ) else {
            throw APIWrongAnswerError.invalidURL
        }

        var request = URLRequest(url: endpoint)
        request.httpMethod = "DELETE"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")

        let (_, response) = try await URLSession.shared.data(for: request)

        guard let http = response as? HTTPURLResponse else {
            throw APIWrongAnswerError.invalidResponse
        }

        guard (200...299).contains(http.statusCode) else {
            if http.statusCode == 401 { throw APIWrongAnswerError.unauthorized }
            throw APIWrongAnswerError.httpError(statusCode: http.statusCode)
        }
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

enum APIWrongAnswerError: LocalizedError, Equatable {
    case invalidURL
    case invalidResponse
    case unauthorized
    case httpError(statusCode: Int)

    var errorDescription: String? {
        switch self {
        case .invalidURL: "Invalid wrong-answers endpoint URL"
        case .invalidResponse: "Invalid response from server"
        case .unauthorized: "Unauthorized (401)"
        case .httpError(let code): "HTTP error: \(code)"
        }
    }
}
