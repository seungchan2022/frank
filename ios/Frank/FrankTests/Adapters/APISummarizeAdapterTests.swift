import Testing
import Foundation
@testable import Frank

/// APISummarizeAdapter — POST /api/me/summarize 검증
/// 타임아웃 70초, URLSession 주입 패턴 확인
@Suite("APISummarizeAdapter Tests", .serialized)
struct APISummarizeAdapterTests {

    // MARK: - Helpers

    private static let testHost = "summarize-test.example.com"

    private func makeAdapter(
        accessToken: String = "test-token",
        getAccessTokenError: Error? = nil
    ) throws -> (APISummarizeAdapter, MockAuthPort) {
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

        let adapter = APISummarizeAdapter(
            auth: auth,
            serverConfig: serverConfig,
            session: session
        )
        return (adapter, auth)
    }

    private func makeResponse(url: URL, statusCode: Int) throws -> HTTPURLResponse {
        try HTTPTestHelpers.makeResponse(url: url, statusCode: statusCode)
    }

    private func summaryResultJSON(
        summary: String = "요약 내용입니다.",
        insight: String = "인사이트 내용입니다."
    ) -> String {
        """
        {
            "summary": "\(summary)",
            "insight": "\(insight)"
        }
        """
    }

    // MARK: - summarize

    @Test("summarize 성공: SummaryResult 디코딩")
    func summarizeSuccess() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()
        let json = summaryResultJSON(summary: "AI 기술 요약", insight: "AI 인사이트")
        guard let data = json.data(using: .utf8) else {
            Issue.record("invalid json fixture")
            return
        }

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            #expect(request.url?.path == "/api/me/summarize")
            #expect(request.httpMethod == "POST")
            #expect(request.value(forHTTPHeaderField: "Authorization") == "Bearer test-token")
            #expect(request.value(forHTTPHeaderField: "Content-Type") == "application/json")
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, data)
        }

        let result = try await adapter.summarize(
            url: "https://example.com/article",
            title: "AI Article"
        )
        #expect(result.summary == "AI 기술 요약")
        #expect(result.insight == "AI 인사이트")
    }

    @Test("summarize 401: unauthorized")
    func summarizeUnauthorized() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 401
            )
            return (response, Data())
        }

        await #expect(throws: APISummarizeError.unauthorized) {
            _ = try await adapter.summarize(url: "https://example.com", title: "Test")
        }
    }

    @Test("summarize 504: timeout 에러 변환")
    func summarizeTimeout() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 504
            )
            return (response, Data())
        }

        await #expect(throws: APISummarizeError.timeout) {
            _ = try await adapter.summarize(url: "https://example.com", title: "Test")
        }
    }

    @Test("summarize 400: badRequest 에러 변환")
    func summarizeBadRequest() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 400
            )
            return (response, Data())
        }

        await #expect(throws: APISummarizeError.badRequest) {
            _ = try await adapter.summarize(url: "https://example.com", title: "Test")
        }
    }

    @Test("토큰 획득 실패: 에러 전파")
    func accessTokenFailure() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter(getAccessTokenError: URLError(.userAuthenticationRequired))

        await #expect(throws: URLError.self) {
            _ = try await adapter.summarize(url: "https://example.com", title: "Test")
        }
    }

    @Test("summarize 요청 본문: url + title 직렬화")
    func summarizeRequestBody() async throws {
        MockURLProtocol.resetHandler(forHost: Self.testHost)
        let (adapter, _) = try makeAdapter()
        let json = summaryResultJSON(summary: "요약", insight: "인사이트")
        guard let data = json.data(using: .utf8) else {
            Issue.record("invalid json fixture")
            return
        }

        MockURLProtocol.setHandler(forHost: Self.testHost) { request in
            // 요청 바디 검증
            if let stream = request.httpBodyStream {
                let body = HTTPTestHelpers.readStream(stream)
                let decoded = try JSONDecoder().decode([String: String].self, from: body)
                #expect(decoded["url"] == "https://example.com/article")
                #expect(decoded["title"] == "Test Title")
            } else if let body = request.httpBody {
                let decoded = try JSONDecoder().decode([String: String].self, from: body)
                #expect(decoded["url"] == "https://example.com/article")
                #expect(decoded["title"] == "Test Title")
            } else {
                Issue.record("expected request body")
            }
            let response = try self.makeResponse(
                url: request.url ?? URL(fileURLWithPath: "/dev/null"),
                statusCode: 200
            )
            return (response, data)
        }

        _ = try await adapter.summarize(url: "https://example.com/article", title: "Test Title")
    }
}
