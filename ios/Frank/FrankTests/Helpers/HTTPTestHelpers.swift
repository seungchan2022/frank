import Foundation

// MARK: - Test Setup Errors

enum TestSetupError: Error {
    case invalidURL
    case invalidResponse
}

/// 어댑터 테스트 공통 HTTP/Stream helper.
///
/// `APITagAdapterTests`, `APIArticleAdapterTests`, `ProfileAPITests`에서 중복으로
/// 정의되던 헬퍼를 단일 출처로 통합.
enum HTTPTestHelpers {

    /// JSON Content-Type 헤더 + 지정한 status code의 HTTPURLResponse.
    /// 실패 시 `TestSetupError.invalidResponse` throw.
    static func makeResponse(url: URL, statusCode: Int) throws -> HTTPURLResponse {
        guard let response = HTTPURLResponse(
            url: url,
            statusCode: statusCode,
            httpVersion: "HTTP/1.1",
            headerFields: ["Content-Type": "application/json"]
        ) else {
            throw TestSetupError.invalidResponse
        }
        return response
    }

    /// `URLRequest.httpBodyStream` 동기 읽기.
    /// 테스트에서 POST/PUT 요청 body 검증에 사용.
    static func readStream(_ stream: InputStream) -> Data {
        stream.open()
        defer { stream.close() }
        var data = Data()
        let bufferSize = 1024
        let buffer = UnsafeMutablePointer<UInt8>.allocate(capacity: bufferSize)
        defer { buffer.deallocate() }
        while stream.hasBytesAvailable {
            let read = stream.read(buffer, maxLength: bufferSize)
            guard read > 0 else { break }
            data.append(buffer, count: read)
        }
        return data
    }
}
