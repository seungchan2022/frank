import Testing
import Foundation
@testable import Frank

@Suite("APITagAdapter Tests", .serialized)
struct APITagAdapterTests {

    // MARK: - Helpers

    private static let testHost = "tag-test.example.com"

    private func makeAdapter(
        accessToken: String = "test-token",
        getAccessTokenError: Error? = nil
    ) throws -> (APITagAdapter, MockAuthPort) {
        let auth = MockAuthPort()
        auth.accessToken = accessToken
        auth.getAccessTokenError = getAccessTokenError

        let config = URLSessionConfiguration.ephemeral
        config.protocolClasses = [MockURLProtocol.self]
        let session = URLSession(configuration: config)

        guard let serverURL = URL(string: "https://\(Self.testHost)") else {
            throw TestSetupError.invalidURL
        }
        let serverConfig = ServerConfig(url: serverURL)

        let adapter = APITagAdapter(
            auth: auth,
            serverConfig: serverConfig,
            session: session
        )
        return (adapter, auth)
    }

    private func makeResponse(url: URL, statusCode: Int) throws -> HTTPURLResponse {
        try HTTPTestHelpers.makeResponse(url: url, statusCode: statusCode)
    }

    // MARK: - fetchAllTags

    @Test("fetchAllTags 성공: TagDTO 배열을 도메인으로 변환")
    func fetchAllTagsSuccess() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()
        let tagId = UUID()
        let json = """
        [{"id":"\(tagId.uuidString)","name":"AI","category":"기술"}]
        """
        guard let data = json.data(using: .utf8) else {
            Issue.record("invalid json fixture")
            return
        }

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            #expect(request.url?.path == "/api/tags")
            #expect(request.httpMethod == "GET")
            #expect(request.value(forHTTPHeaderField: "Authorization") == "Bearer test-token")
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, data)
        }

        let tags = try await adapter.fetchAllTags()
        #expect(tags.count == 1)
        #expect(tags[0].id == tagId)
        #expect(tags[0].name == "AI")
        #expect(tags[0].category == "기술")
    }

    @Test("fetchAllTags: category nullable 디코딩")
    func fetchAllTagsCategoryNull() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()
        let tagId = UUID()
        let json = """
        [{"id":"\(tagId.uuidString)","name":"디자인","category":null}]
        """
        guard let data = json.data(using: .utf8) else {
            Issue.record("invalid json fixture")
            return
        }

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, data)
        }

        let tags = try await adapter.fetchAllTags()
        #expect(tags.count == 1)
        #expect(tags[0].category == nil)
    }

    @Test("fetchAllTags 401: APITagError.unauthorized")
    func fetchAllTagsUnauthorized() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 401
            )
            return (response, Data())
        }

        await #expect(throws: APITagError.unauthorized) {
            _ = try await adapter.fetchAllTags()
        }
    }

    // MARK: - fetchMyTagIds

    @Test("fetchMyTagIds 성공: UUID 문자열 배열 파싱")
    func fetchMyTagIdsSuccess() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()
        let id1 = UUID()
        let id2 = UUID()
        let json = "[\"\(id1.uuidString)\",\"\(id2.uuidString)\"]"
        guard let data = json.data(using: .utf8) else {
            Issue.record("invalid json fixture")
            return
        }

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            #expect(request.url?.path == "/api/me/tags")
            #expect(request.httpMethod == "GET")
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, data)
        }

        let ids = try await adapter.fetchMyTagIds()
        #expect(ids == [id1, id2])
    }

    @Test("fetchMyTagIds: 잘못된 UUID 응답 시 invalidUUIDInResponse throw (silent drop 차단)")
    func fetchMyTagIdsMalformed() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()
        let json = "[\"\(UUID().uuidString)\",\"NOT-A-UUID\"]"
        guard let data = json.data(using: .utf8) else {
            Issue.record("invalid json fixture")
            return
        }

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, data)
        }

        await #expect(throws: APITagError.invalidUUIDInResponse("NOT-A-UUID")) {
            _ = try await adapter.fetchMyTagIds()
        }
    }

    @Test("fetchMyTagIds 401: unauthorized")
    func fetchMyTagIdsUnauthorized() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 401
            )
            return (response, Data())
        }

        await #expect(throws: APITagError.unauthorized) {
            _ = try await adapter.fetchMyTagIds()
        }
    }

    // MARK: - saveMyTags

    @Test("saveMyTags 성공: POST + tag_ids body 직렬화")
    func saveMyTagsSuccess() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()
        let id1 = UUID()
        let id2 = UUID()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            #expect(request.url?.path == "/api/me/tags")
            #expect(request.httpMethod == "POST")
            #expect(request.value(forHTTPHeaderField: "Content-Type") == "application/json")
            // bodyStreamData 검증
            if let stream = request.httpBodyStream {
                let body = Self.readStream(stream)
                let decoded = try JSONDecoder().decode([String: [String]].self, from: body)
                #expect(decoded["tag_ids"]?.count == 2)
                #expect(decoded["tag_ids"]?.contains(id1.uuidString) == true)
                #expect(decoded["tag_ids"]?.contains(id2.uuidString) == true)
            } else if let body = request.httpBody {
                let decoded = try JSONDecoder().decode([String: [String]].self, from: body)
                #expect(decoded["tag_ids"]?.count == 2)
            } else {
                Issue.record("expected request body")
            }

            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, Data("{\"ok\":true}".utf8))
        }

        try await adapter.saveMyTags(tagIds: [id1, id2])
    }

    @Test("saveMyTags 401: unauthorized")
    func saveMyTagsUnauthorized() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 401
            )
            return (response, Data())
        }

        await #expect(throws: APITagError.unauthorized) {
            try await adapter.saveMyTags(tagIds: [UUID()])
        }
    }

    @Test("토큰 획득 실패: 에러 전파")
    func accessTokenFailure() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter(getAccessTokenError: URLError(.userAuthenticationRequired))

        await #expect(throws: URLError.self) {
            _ = try await adapter.fetchAllTags()
        }
    }

    // MARK: - Stream helper (delegates to HTTPTestHelpers)

    private static func readStream(_ stream: InputStream) -> Data {
        HTTPTestHelpers.readStream(stream)
    }
}
