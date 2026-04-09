import Testing
import Foundation
@testable import Frank

/// MVP5 M1: APIArticleAdapter — GET /api/me/feed 검증
@Suite("APIArticleAdapter Tests", .serialized)
struct APIArticleAdapterTests {

    // MARK: - Helpers

    private static let testHost = "article-test.example.com"

    private func makeAdapter(
        accessToken: String = "test-token",
        getAccessTokenError: Error? = nil
    ) throws -> (APIArticleAdapter, MockAuthPort) {
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

        let adapter = APIArticleAdapter(
            auth: auth,
            serverConfig: serverConfig,
            session: session
        )
        return (adapter, auth)
    }

    private func makeResponse(url: URL, statusCode: Int) throws -> HTTPURLResponse {
        try HTTPTestHelpers.makeResponse(url: url, statusCode: statusCode)
    }

    private func feedItemJSON(
        title: String = "Hello",
        url: String = "https://example.com/article",
        source: String = "tavily",
        tagId: UUID? = nil
    ) -> String {
        let tagFragment = tagId.map { "\"\($0.uuidString)\"" } ?? "null"
        return """
        {
            "title": "\(title)",
            "url": "\(url)",
            "snippet": "snippet text",
            "source": "\(source)",
            "published_at": "2026-04-05T14:00:00Z",
            "tag_id": \(tagFragment)
        }
        """
    }

    // MARK: - fetchFeed 성공

    @Test("fetchFeed 성공: GET /api/me/feed + FeedItem 디코딩")
    func fetchFeedSuccess() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()
        let tagId = UUID()
        let json = "[\(feedItemJSON(title: "Hello", tagId: tagId))]"
        guard let data = json.data(using: .utf8) else {
            Issue.record("invalid json")
            return
        }

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            #expect(request.url?.path == "/api/me/feed")
            #expect(request.httpMethod == "GET")
            #expect(request.value(forHTTPHeaderField: "Authorization") == "Bearer test-token")

            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, data)
        }

        let items = try await adapter.fetchFeed()
        #expect(items.count == 1)
        #expect(items[0].title == "Hello")
        #expect(items[0].source == "tavily")
        #expect(items[0].tagId == tagId)
        #expect(items[0].snippet == "snippet text")
    }

    // MARK: - fetchFeed 빈 배열

    @Test("fetchFeed 빈 배열 응답: 빈 [FeedItem] 반환")
    func fetchFeedEmpty() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, Data("[]".utf8))
        }

        let items = try await adapter.fetchFeed()
        #expect(items.isEmpty)
    }

    // MARK: - fetchFeed URL 파싱

    @Test("fetchFeed: 깨진 url 응답 시 invalidArticleURL throw")
    func fetchFeedMalformedURL() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()

        let json = """
        [{
            "title": "Hello",
            "url": "ht!tp:// broken",
            "snippet": null,
            "source": "tavily",
            "published_at": null,
            "tag_id": null
        }]
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

        await #expect(throws: APIArticleError.self) {
            _ = try await adapter.fetchFeed()
        }
    }

    // MARK: - fetchFeed 날짜 디코딩

    @Test("fetchFeed: microseconds(6자리) timestamp 디코딩")
    func fetchFeedMicrosecondTimestamp() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()

        let json = """
        [{
            "title": "Hello",
            "url": "https://example.com",
            "snippet": null,
            "source": "tavily",
            "published_at": "2026-04-07T07:32:37.350714Z",
            "tag_id": null
        }]
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

        let items = try await adapter.fetchFeed()
        #expect(items.count == 1)
        #expect(items[0].publishedAt != nil)
    }

    // MARK: - 에러 처리

    @Test("fetchFeed 401: unauthorized")
    func fetchFeedUnauthorized() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 401
            )
            return (response, Data())
        }

        await #expect(throws: APIArticleError.unauthorized) {
            _ = try await adapter.fetchFeed()
        }
    }

    @Test("fetchFeed 400: httpError 전파")
    func fetchFeedBadRequest() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 400
            )
            return (response, Data())
        }

        await #expect(throws: APIArticleError.httpError(statusCode: 400)) {
            _ = try await adapter.fetchFeed()
        }
    }

    @Test("토큰 획득 실패 전파")
    func accessTokenFailure() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter(getAccessTokenError: URLError(.userAuthenticationRequired))

        await #expect(throws: URLError.self) {
            _ = try await adapter.fetchFeed()
        }
    }
}
