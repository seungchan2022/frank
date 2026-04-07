import Foundation

/// 테스트용 URLProtocol — URLSession HTTP 호출을 가로채 결정적 응답을 주입한다.
///
/// 핸들러는 **호스트별로 격리**되어 다른 테스트 suite와 병렬 실행되어도 충돌하지 않는다.
/// 각 suite는 고유 호스트(예: `tag-test.example.com`)를 사용하라.
///
/// 사용법:
/// ```swift
/// let config = URLSessionConfiguration.ephemeral
/// config.protocolClasses = [MockURLProtocol.self]
/// let session = URLSession(configuration: config)
///
/// MockURLProtocol.setHandler(forHost: "tag-test.example.com") { request in
///     let response = HTTPURLResponse(...)
///     return (response, jsonData)
/// }
/// ```
final class MockURLProtocol: URLProtocol, @unchecked Sendable {
    typealias Handler = @Sendable (URLRequest) throws -> (HTTPURLResponse, Data)

    private static let lock = NSLock()
    nonisolated(unsafe) private static var handlers: [String: Handler] = [:]

    static func setHandler(forHost host: String, _ handler: @escaping Handler) {
        lock.lock()
        defer { lock.unlock() }
        handlers[host] = handler
    }

    static func resetHandler(forHost host: String) {
        lock.lock()
        defer { lock.unlock() }
        handlers.removeValue(forKey: host)
    }

    static func resetAll() {
        lock.lock()
        defer { lock.unlock() }
        handlers.removeAll()
    }

    private static func handler(forHost host: String?) -> Handler? {
        guard let host else { return nil }
        lock.lock()
        defer { lock.unlock() }
        return handlers[host]
    }

    // swiftlint:disable:next static_over_final_class
    override class func canInit(with request: URLRequest) -> Bool { true }

    // swiftlint:disable:next static_over_final_class
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }

    override func startLoading() {
        guard let handler = MockURLProtocol.handler(forHost: request.url?.host) else {
            client?.urlProtocol(self, didFailWithError: URLError(.cannotConnectToHost))
            return
        }
        do {
            let (response, data) = try handler(request)
            client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
            client?.urlProtocol(self, didLoad: data)
            client?.urlProtocolDidFinishLoading(self)
        } catch {
            client?.urlProtocol(self, didFailWithError: error)
        }
    }

    override func stopLoading() {}
}
