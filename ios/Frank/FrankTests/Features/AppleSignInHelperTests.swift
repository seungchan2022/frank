import Testing
import Foundation
import CryptoKit
@testable import Frank

@Suite("AppleSignInHelper Tests")
struct AppleSignInHelperTests {

    // MARK: - randomNonce

    @Test("randomNonce 기본 길이는 32바이트이며 유효 문자만 포함")
    func randomNonceDefaultLength() throws {
        let nonce = try AppleSignInHelper.randomNonce()

        // 32바이트 → charset 인코딩 후 동일 길이 (1바이트당 1문자)
        #expect(nonce.count == 32)
    }

    @Test("randomNonce 커스텀 길이 동작")
    func randomNonceCustomLength() throws {
        let nonce = try AppleSignInHelper.randomNonce(length: 64)
        #expect(nonce.count == 64)
    }

    @Test("randomNonce — 허용 문자셋만 포함 (영숫자 + -._)")
    func randomNonceAllowedCharset() throws {
        let nonce = try AppleSignInHelper.randomNonce(length: 128)

        // charset: "0123456789ABCDEFGHIJKLMNOPQRSTUVXYZabcdefghijklmnopqrstuvwxyz-._"
        // 특수문자: -, ., _ 만 허용
        let allowedPattern = "^[0-9A-Za-z._-]+$"
        let regex = try NSRegularExpression(pattern: allowedPattern)
        let range = NSRange(nonce.startIndex..., in: nonce)
        let matches = regex.numberOfMatches(in: nonce, range: range)

        #expect(matches == 1, "nonce에 허용되지 않은 문자 포함: \(nonce)")
    }

    @Test("randomNonce 두 번 호출 시 다른 값 반환 (충분한 엔트로피)")
    func randomNonceIsRandom() throws {
        let nonce1 = try AppleSignInHelper.randomNonce()
        let nonce2 = try AppleSignInHelper.randomNonce()
        #expect(nonce1 != nonce2)
    }

    // MARK: - sha256

    @Test("sha256 known vector: SHA-256(\"hello\") = 2cf24dba...")
    func sha256KnownVector() {
        // known SHA-256("hello") hex = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        let result = AppleSignInHelper.sha256("hello")
        #expect(result == "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824")
    }

    @Test("sha256 빈 문자열 known vector")
    func sha256EmptyString() {
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let result = AppleSignInHelper.sha256("")
        #expect(result == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
    }

    @Test("sha256 출력 길이는 항상 64자 (hex encoded 256bit)")
    func sha256OutputLength() {
        let inputs = ["hello", "world", "nonce123", "a", ""]
        for input in inputs {
            let result = AppleSignInHelper.sha256(input)
            #expect(result.count == 64, "입력: \(input), 길이: \(result.count)")
        }
    }

    @Test("sha256 출력은 소문자 hex 문자만 포함")
    func sha256HexCharsOnly() throws {
        let result = AppleSignInHelper.sha256("test-nonce-value")
        let hexPattern = "^[0-9a-f]+$"
        let regex = try NSRegularExpression(pattern: hexPattern)
        let range = NSRange(result.startIndex..., in: result)
        #expect(regex.numberOfMatches(in: result, range: range) == 1)
    }
}
