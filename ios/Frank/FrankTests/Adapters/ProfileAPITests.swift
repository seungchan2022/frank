import Testing
import Foundation
@testable import Frank

@Suite("ProfileAPI Tests", .serialized)
struct ProfileAPITests {

    // MARK: - Helpers

    private static let testHost = "profile-test.example.com"

    private func makeSession() -> URLSession {
        let config = URLSessionConfiguration.ephemeral
        config.protocolClasses = [MockURLProtocol.self]
        return URLSession(configuration: config)
    }

    private func makeServerURL() throws -> URL {
        guard let url = URL(string: "https://\(Self.testHost)") else {
            throw TestSetupError.invalidURL
        }
        return url
    }

    private func makeResponse(url: URL, statusCode: Int) throws -> HTTPURLResponse {
        try HTTPTestHelpers.makeResponse(url: url, statusCode: statusCode)
    }

    // MARK: - Happy path

    @Test("updateProfile 성공: PUT + body 직렬화 + Profile 디코딩")
    func updateProfileSuccess() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let session = makeSession()
        let serverURL = try makeServerURL()
        let profileId = UUID()

        let responseJSON = """
        {"id":"\(profileId.uuidString)","display_name":"홍길동","onboarding_completed":true}
        """
        guard let data = responseJSON.data(using: .utf8) else {
            Issue.record("invalid json")
            return
        }

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            #expect(request.url?.path == "/api/me/profile")
            #expect(request.httpMethod == "PUT")
            #expect(request.value(forHTTPHeaderField: "Authorization") == "Bearer test-token")
            #expect(request.value(forHTTPHeaderField: "Content-Type") == "application/json")

            // body에 onboarding_completed: true 포함 검증
            if let stream = request.httpBodyStream {
                let body = Self.readStream(stream)
                let decoded = try JSONSerialization.jsonObject(with: body) as? [String: Any]
                #expect(decoded?["onboarding_completed"] as? Bool == true)
            }

            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, data)
        }

        let profile = try await ProfileAPI.updateProfile(
            token: "test-token",
            onboardingCompleted: true,
            serverURL: serverURL,
            session: session
        )

        #expect(profile.id == profileId)
        #expect(profile.displayName == "홍길동")
        #expect(profile.onboardingCompleted == true)
    }

    @Test("updateProfile displayName만: 빈 onboarding 키 생략")
    func updateProfileDisplayNameOnly() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let session = makeSession()
        let serverURL = try makeServerURL()
        let profileId = UUID()

        let responseJSON = """
        {"id":"\(profileId.uuidString)","display_name":"새이름","onboarding_completed":false}
        """
        guard let data = responseJSON.data(using: .utf8) else {
            Issue.record("invalid json")
            return
        }

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            if let stream = request.httpBodyStream {
                let body = Self.readStream(stream)
                let decoded = try JSONSerialization.jsonObject(with: body) as? [String: Any]
                #expect(decoded?["display_name"] as? String == "새이름")
                #expect(decoded?["onboarding_completed"] == nil)
            }

            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, data)
        }

        let profile = try await ProfileAPI.updateProfile(
            token: "test-token",
            displayName: "새이름",
            serverURL: serverURL,
            session: session
        )

        #expect(profile.displayName == "새이름")
    }

    // MARK: - fetchProfile (M3 ST-5 fix — currentSession 진실의 원천)

    @Test("fetchProfile 성공: GET /api/me/profile + Profile 디코딩")
    func fetchProfileSuccess() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let session = makeSession()
        let serverURL = try makeServerURL()
        let profileId = UUID()

        let responseJSON = """
        {"id":"\(profileId.uuidString)","display_name":"홍길동","onboarding_completed":true}
        """
        guard let data = responseJSON.data(using: .utf8) else {
            Issue.record("invalid json")
            return
        }

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            #expect(request.url?.path == "/api/me/profile")
            #expect(request.httpMethod == "GET")
            #expect(request.value(forHTTPHeaderField: "Authorization") == "Bearer test-token")

            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, data)
        }

        let profile = try await ProfileAPI.fetchProfile(
            token: "test-token",
            serverURL: serverURL,
            session: session
        )
        #expect(profile.id == profileId)
        #expect(profile.displayName == "홍길동")
        #expect(profile.onboardingCompleted == true)
    }

    @Test("fetchProfile 404: ProfileAPIError.notFound (신규 가입 race)")
    func fetchProfileNotFound() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let session = makeSession()
        let serverURL = try makeServerURL()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 404
            )
            return (response, Data())
        }

        await #expect(throws: ProfileAPIError.notFound) {
            _ = try await ProfileAPI.fetchProfile(
                token: "test-token",
                serverURL: serverURL,
                session: session
            )
        }
    }

    @Test("fetchProfile 401: ProfileAPIError.unauthorized")
    func fetchProfileUnauthorized() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let session = makeSession()
        let serverURL = try makeServerURL()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 401
            )
            return (response, Data())
        }

        await #expect(throws: ProfileAPIError.unauthorized) {
            _ = try await ProfileAPI.fetchProfile(
                token: "bad",
                serverURL: serverURL,
                session: session
            )
        }
    }

    // MARK: - 부정 테스트 (필수, 0_CODEX_RULES §4.2)

    @Test("updateProfile 401: ProfileAPIError.unauthorized")
    func updateProfileUnauthorized() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let session = makeSession()
        let serverURL = try makeServerURL()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 401
            )
            return (response, Data())
        }

        await #expect(throws: ProfileAPIError.unauthorized) {
            _ = try await ProfileAPI.updateProfile(
                token: "bad-token",
                onboardingCompleted: true,
                serverURL: serverURL,
                session: session
            )
        }
    }

    @Test("updateProfile 400: ProfileAPIError.badRequest")
    func updateProfileBadRequest() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let session = makeSession()
        let serverURL = try makeServerURL()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 400
            )
            return (response, Data())
        }

        await #expect(throws: ProfileAPIError.badRequest) {
            _ = try await ProfileAPI.updateProfile(
                token: "test-token",
                displayName: "",
                serverURL: serverURL,
                session: session
            )
        }
    }

    @Test("updateProfile 200 + 잘못된 JSON: decodingFailed")
    func updateProfileDecodingFailed() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let session = makeSession()
        let serverURL = try makeServerURL()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, Data("{\"garbage\":true}".utf8))
        }

        await #expect(throws: ProfileAPIError.decodingFailed) {
            _ = try await ProfileAPI.updateProfile(
                token: "test-token",
                onboardingCompleted: true,
                serverURL: serverURL,
                session: session
            )
        }
    }

    @Test("updateProfile 500: httpError")
    func updateProfileServerError() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let session = makeSession()
        let serverURL = try makeServerURL()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 500
            )
            return (response, Data())
        }

        await #expect(throws: ProfileAPIError.httpError(statusCode: 500)) {
            _ = try await ProfileAPI.updateProfile(
                token: "test-token",
                onboardingCompleted: true,
                serverURL: serverURL,
                session: session
            )
        }
    }

    // MARK: - Stream helper (delegates to HTTPTestHelpers)

    private static func readStream(_ stream: InputStream) -> Data {
        HTTPTestHelpers.readStream(stream)
    }
}
