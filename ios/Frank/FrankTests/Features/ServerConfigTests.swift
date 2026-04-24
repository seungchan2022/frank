import Testing
@testable import Frank

@Suite("ServerConfig")
struct ServerConfigTests {

    // MARK: - make(urlString:)

    @Test("유효한 URL 문자열로 ServerConfig 생성 성공")
    func makeWithValidURL() throws {
        let config = try ServerConfig.make(urlString: "http://localhost:8080")
        #expect(config.url.absoluteString == "http://localhost:8080")
    }

    @Test("빈 문자열로 make 호출 시 invalidURL 에러 발생")
    func makeWithEmptyStringThrows() throws {
        #expect(throws: ServerConfigError.invalidURL("")) {
            try ServerConfig.make(urlString: "")
        }
    }

    @Test("스킴 없는 문자열은 URL로 파싱되므로 에러 미발생 (Swift URL 동작)")
    func makeWithNoSchemeDoesNotThrow() throws {
        // Swift의 URL(string:)은 상대 경로도 유효한 URL로 처리함
        // make()는 URL 파싱 성공 여부만 체크하므로 이 케이스는 성공
        let config = try ServerConfig.make(urlString: "localhost:8080")
        #expect(config.url.absoluteString == "localhost:8080")
    }

    @Test("유효한 HTTPS URL로 ServerConfig 생성 성공")
    func makeWithHTTPSURL() throws {
        let config = try ServerConfig.make(urlString: "https://api.example.com")
        #expect(config.url.scheme == "https")
        #expect(config.url.host() == "api.example.com")
    }

    // MARK: - ServerConfigError Equatable

    @Test("ServerConfigError.invalidURL 동등성 비교")
    func serverConfigErrorEquality() {
        let err1 = ServerConfigError.invalidURL("bad")
        let err2 = ServerConfigError.invalidURL("bad")
        let err3 = ServerConfigError.invalidURL("other")
        #expect(err1 == err2)
        #expect(err1 != err3)
    }

    @Test("ServerConfigError.missing은 invalidURL과 다름")
    func serverConfigErrorMissingDistinct() {
        #expect(ServerConfigError.missing != .invalidURL(""))
        #expect(ServerConfigError.missing == .missing)
    }

    // MARK: - live() — 시뮬레이터 환경 (컴파일 타임 분기)

    @Test("시뮬레이터에서 live()는 localhost:8080 반환")
    func liveReturnsLocalhostInSimulator() throws {
        #if targetEnvironment(simulator)
        let config = try ServerConfig.live()
        #expect(config.url.absoluteString == "http://localhost:8080")
        #else
        // 실기기 테스트 환경에서는 xcconfig나 Secrets.plist 없이 에러가 발생할 수 있음
        // 이 경로는 실기기 CI에서 별도 검증
        #expect(Bool(true)) // 실기기에서는 이 케이스 스킵
        #endif
    }
}
