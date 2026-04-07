import Foundation

/// Rust API의 `/api/me/profile` 호출 helper.
///
/// SupabaseAuthAdapter가 profile 읽기/쓰기를 user_metadata 직접 접근 대신 이 helper로 위임한다.
/// **진실의 원천은 server `profiles` 테이블** — Supabase user_metadata는 사용 금지
/// (두 데이터 출처가 분리되면 일관성 결함 발생, M3 ST-5 fix 사례 참고).
///
/// 테스트 가능성을 위해 SupabaseClient 의존성과 분리된 pure helper로 둔다.
enum ProfileAPI {

    /// `GET /api/me/profile` 호출 — server profiles 테이블에서 진실의 원천 fetch.
    static func fetchProfile(
        token: String,
        serverURL: URL,
        session: URLSession
    ) async throws -> Profile {
        guard let url = URL(string: "/api/me/profile", relativeTo: serverURL) else {
            throw ProfileAPIError.invalidURL
        }
        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")

        let (data, response) = try await session.data(for: request)
        guard let http = response as? HTTPURLResponse else {
            throw ProfileAPIError.invalidResponse
        }
        switch http.statusCode {
        case 200...299:
            do {
                let dto = try JSONDecoder().decode(ProfileDTO.self, from: data)
                return dto.toDomain()
            } catch {
                throw ProfileAPIError.decodingFailed
            }
        case 401:
            throw ProfileAPIError.unauthorized
        case 404:
            throw ProfileAPIError.notFound
        default:
            throw ProfileAPIError.httpError(statusCode: http.statusCode)
        }
    }

    /// `PUT /api/me/profile` 호출.
    /// - Parameters:
    ///   - token: Supabase access token (Bearer)
    ///   - displayName: 갱신할 표시명. nil이면 변경 안 함
    ///   - onboardingCompleted: 갱신할 onboarding 플래그. nil이면 변경 안 함
    ///   - serverURL: Rust API 서버 베이스 URL
    ///   - session: URLSession (테스트 시 URLProtocol mock 주입)
    static func updateProfile(
        token: String,
        displayName: String? = nil,
        onboardingCompleted: Bool? = nil,
        serverURL: URL,
        session: URLSession
    ) async throws -> Profile {
        guard let url = URL(string: "/api/me/profile", relativeTo: serverURL) else {
            throw ProfileAPIError.invalidURL
        }
        var request = URLRequest(url: url)
        request.httpMethod = "PUT"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let body = ProfileUpdateRequest(
            displayName: displayName,
            onboardingCompleted: onboardingCompleted
        )
        request.httpBody = try JSONEncoder().encode(body)

        let (data, response) = try await session.data(for: request)
        guard let http = response as? HTTPURLResponse else {
            throw ProfileAPIError.invalidResponse
        }
        switch http.statusCode {
        case 200...299:
            do {
                let dto = try JSONDecoder().decode(ProfileDTO.self, from: data)
                return dto.toDomain()
            } catch {
                throw ProfileAPIError.decodingFailed
            }
        case 401:
            throw ProfileAPIError.unauthorized
        case 400:
            throw ProfileAPIError.badRequest
        default:
            throw ProfileAPIError.httpError(statusCode: http.statusCode)
        }
    }
}

// MARK: - DTOs

struct ProfileDTO: Decodable {
    let id: UUID
    let displayName: String?
    let onboardingCompleted: Bool

    enum CodingKeys: String, CodingKey {
        case id
        case displayName = "display_name"
        case onboardingCompleted = "onboarding_completed"
    }

    func toDomain() -> Profile {
        Profile(
            id: id,
            displayName: displayName,
            onboardingCompleted: onboardingCompleted
        )
    }
}

private struct ProfileUpdateRequest: Encodable {
    let displayName: String?
    let onboardingCompleted: Bool?

    enum CodingKeys: String, CodingKey {
        case displayName = "display_name"
        case onboardingCompleted = "onboarding_completed"
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        // 필드가 nil이면 키 자체를 생략 (server 빈 바디 = no-op 정책)
        try container.encodeIfPresent(displayName, forKey: .displayName)
        try container.encodeIfPresent(onboardingCompleted, forKey: .onboardingCompleted)
    }
}

// MARK: - Errors

enum ProfileAPIError: LocalizedError, Equatable {
    case invalidURL
    case invalidResponse
    case unauthorized
    case badRequest
    case notFound
    case decodingFailed
    case httpError(statusCode: Int)

    var errorDescription: String? {
        switch self {
        case .invalidURL: "Invalid profile API URL"
        case .invalidResponse: "Invalid response from server"
        case .unauthorized: "Unauthorized (401)"
        case .badRequest: "Bad request (400)"
        case .notFound: "Profile not found (404)"
        case .decodingFailed: "Failed to decode profile response"
        case .httpError(let code): "HTTP error: \(code)"
        }
    }
}
