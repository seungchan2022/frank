import Testing
import Foundation
@testable import Frank

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

    private func articleJSON(id: UUID, userId: UUID, tagId: UUID?) -> String {
        let tagFragment: String
        if let tagId {
            tagFragment = "\"\(tagId.uuidString)\""
        } else {
            tagFragment = "null"
        }
        return """
        {
            "id": "\(id.uuidString)",
            "user_id": "\(userId.uuidString)",
            "tag_id": \(tagFragment),
            "title": "Hello",
            "url": "https://example.com/article",
            "snippet": "snippet",
            "source": "tavily",
            "published_at": "2026-04-05T14:00:00Z",
            "created_at": "2026-04-06T09:00:00Z"
        }
        """
    }

    // MARK: - fetchArticles

    @Test("fetchArticles 성공: limit/offset 쿼리 + 디코딩")
    func fetchArticlesSuccess() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()
        let articleId = UUID()
        let userId = UUID()
        let json = "[\(articleJSON(id: articleId, userId: userId, tagId: nil))]"
        guard let data = json.data(using: .utf8) else {
            Issue.record("invalid json")
            return
        }

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            #expect(request.url?.path == "/api/me/articles")
            #expect(request.httpMethod == "GET")
            #expect(request.value(forHTTPHeaderField: "Authorization") == "Bearer test-token")
            let queryItems = URLComponents(url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                                           resolvingAgainstBaseURL: false)?.queryItems ?? []
            #expect(queryItems.contains { $0.name == "limit" && $0.value == "20" })
            #expect(queryItems.contains { $0.name == "offset" && $0.value == "0" })
            #expect(!queryItems.contains { $0.name == "tag_id" })

            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, data)
        }

        let articles = try await adapter.fetchArticles(filter: ArticleFilter())
        #expect(articles.count == 1)
        #expect(articles[0].id == articleId)
        #expect(articles[0].userId == userId)
        #expect(articles[0].title == "Hello")
        #expect(articles[0].snippet == "snippet")
    }

    @Test("fetchArticles tagId 포함: tag_id 쿼리 추가")
    func fetchArticlesWithTagId() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()
        let tagId = UUID()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let queryItems = URLComponents(url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                                           resolvingAgainstBaseURL: false)?.queryItems ?? []
            #expect(queryItems.contains { $0.name == "tag_id" && $0.value == tagId.uuidString })

            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, Data("[]".utf8))
        }

        let result = try await adapter.fetchArticles(
            filter: ArticleFilter(tagId: tagId, limit: 50, offset: 10)
        )
        #expect(result.isEmpty)
    }

    @Test("fetchArticles: 잘못된 article url 응답 시 invalidArticleURL throw (fallback 금지)")
    func fetchArticlesMalformedURL() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()
        let articleId = UUID()
        let userId = UUID()
        // 의도적으로 깨진 url ("not a url")
        let json = """
        [{
            "id": "\(articleId.uuidString)",
            "user_id": "\(userId.uuidString)",
            "tag_id": null,
            "title": "Hello",
            "url": "ht!tp:// broken",
            "snippet": null,
            "source": "tavily",
            "published_at": null,
            "created_at": null
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

        // 깨진 url을 silent fallback 하지 않고 throw
        await #expect(throws: APIArticleError.self) {
            _ = try await adapter.fetchArticles(filter: ArticleFilter())
        }
    }

    @Test("fetchArticles: PostgREST microseconds(6자리) timestamp 디코딩")
    func fetchArticlesMicrosecondTimestamp() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()
        let articleId = UUID()
        let userId = UUID()
        // PostgREST가 반환하는 실 형식: microseconds 6자리
        let json = """
        [{
            "id": "\(articleId.uuidString)",
            "user_id": "\(userId.uuidString)",
            "tag_id": null,
            "title": "Hello",
            "url": "https://example.com",
            "snippet": null,
            "source": "tavily",
            "published_at": null,
            "created_at": "2026-04-07T07:32:37.350714Z"
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

        let articles = try await adapter.fetchArticles(filter: ArticleFilter())
        #expect(articles.count == 1)
        #expect(articles[0].createdAt != nil)
    }

    @Test("fetchArticles 401: unauthorized")
    func fetchArticlesUnauthorized() async throws {
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
            _ = try await adapter.fetchArticles(filter: ArticleFilter())
        }
    }

    @Test("fetchArticles 400: httpError 전파")
    func fetchArticlesBadRequest() async throws {
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
            _ = try await adapter.fetchArticles(filter: ArticleFilter())
        }
    }

    // MARK: - fetchArticle

    @Test("fetchArticle 성공: path에 id 포함 + 단건 디코딩")
    func fetchArticleSuccess() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()
        let articleId = UUID()
        let userId = UUID()
        let json = articleJSON(id: articleId, userId: userId, tagId: nil)
        guard let data = json.data(using: .utf8) else {
            Issue.record("invalid json")
            return
        }

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            #expect(request.url?.path == "/api/me/articles/\(articleId.uuidString)")
            #expect(request.httpMethod == "GET")

            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, data)
        }

        let article = try await adapter.fetchArticle(id: articleId)
        #expect(article.id == articleId)
        #expect(article.userId == userId)
    }

    @Test("fetchArticle 404: notFound")
    func fetchArticleNotFound() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 404
            )
            return (response, Data())
        }

        await #expect(throws: APIArticleError.notFound) {
            _ = try await adapter.fetchArticle(id: UUID())
        }
    }

    @Test("fetchArticle 401: unauthorized")
    func fetchArticleUnauthorized() async throws {
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
            _ = try await adapter.fetchArticle(id: UUID())
        }
    }

    @Test("토큰 획득 실패 전파")
    func accessTokenFailure() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter(getAccessTokenError: URLError(.userAuthenticationRequired))

        await #expect(throws: URLError.self) {
            _ = try await adapter.fetchArticles(filter: ArticleFilter())
        }
    }
}
