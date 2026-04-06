import Testing
import Foundation
@testable import Frank

@Suite("Port Contract Tests")
struct PortContractTests {

    // MARK: - AuthPort

    @Test("MockAuthPort signIn 성공")
    func authSignInSuccess() async throws {
        let mock = MockAuthPort()
        let profile = Profile(id: UUID(), email: "user@test.com", onboardingCompleted: true)
        mock.signInResult = .success(profile)

        let result = try await mock.signIn(email: "user@test.com", password: "pass")

        #expect(result == profile)
        #expect(mock.signInCallCount == 1)
    }

    @Test("MockAuthPort signIn 실패")
    func authSignInFailure() async {
        let mock = MockAuthPort()
        mock.signInResult = .failure(URLError(.userAuthenticationRequired))

        await #expect(throws: URLError.self) {
            try await mock.signIn(email: "bad", password: "bad")
        }
    }

    @Test("MockAuthPort signUp 성공 (Profile 반환)")
    func authSignUpSuccess() async throws {
        let mock = MockAuthPort()
        let profile = Profile(id: UUID(), email: "new@test.com", onboardingCompleted: false)
        mock.signUpResult = .success(profile)

        let result = try await mock.signUp(email: "new@test.com", password: "pass")

        #expect(result == profile)
        #expect(mock.signUpCallCount == 1)
    }

    @Test("MockAuthPort signUp 이메일 확인 대기 (nil 반환)")
    func authSignUpPendingConfirmation() async throws {
        let mock = MockAuthPort()
        mock.signUpResult = .success(nil)

        let result = try await mock.signUp(email: "new@test.com", password: "pass")

        #expect(result == nil)
        #expect(mock.signUpCallCount == 1)
    }

    @Test("MockAuthPort signInWithApple 성공")
    func authSignInWithAppleSuccess() async throws {
        let mock = MockAuthPort()
        let profile = Profile(id: UUID(), email: "apple@test.com", onboardingCompleted: false)
        mock.signInWithAppleResult = .success(profile)

        let result = try await mock.signInWithApple(idToken: "token", rawNonce: "nonce")

        #expect(result == profile)
        #expect(mock.signInWithAppleCallCount == 1)
    }

    @Test("MockAuthPort signInWithApple 실패")
    func authSignInWithAppleFailure() async {
        let mock = MockAuthPort()
        mock.signInWithAppleResult = .failure(URLError(.userAuthenticationRequired))

        await #expect(throws: URLError.self) {
            try await mock.signInWithApple(idToken: "bad", rawNonce: "bad")
        }
    }

    @Test("MockAuthPort signOut 에러 전파")
    func authSignOutError() async {
        let mock = MockAuthPort()
        mock.signOutError = URLError(.networkConnectionLost)

        await #expect(throws: URLError.self) {
            try await mock.signOut()
        }
        #expect(mock.signOutCallCount == 1)
    }

    @Test("MockAuthPort currentSession nil 반환")
    func authCurrentSessionNil() async throws {
        let mock = MockAuthPort()
        mock.currentSessionResult = nil

        let result = try await mock.currentSession()

        #expect(result == nil)
    }

    @Test("MockAuthPort updateOnboardingCompleted 성공")
    func authUpdateOnboardingSuccess() async throws {
        let mock = MockAuthPort()
        let profile = Profile(id: UUID(), email: "user@test.com", onboardingCompleted: true)
        mock.updateOnboardingCompletedResult = .success(profile)

        let result = try await mock.updateOnboardingCompleted()

        #expect(result == profile)
        #expect(result.onboardingCompleted == true)
        #expect(mock.updateOnboardingCompletedCallCount == 1)
    }

    @Test("MockAuthPort updateOnboardingCompleted 실패")
    func authUpdateOnboardingFailure() async {
        let mock = MockAuthPort()
        mock.updateOnboardingCompletedResult = .failure(URLError(.badServerResponse))

        await #expect(throws: URLError.self) {
            try await mock.updateOnboardingCompleted()
        }
        #expect(mock.updateOnboardingCompletedCallCount == 1)
    }

    // MARK: - TagPort

    @Test("MockTagPort 태그 목록 반환")
    func tagFetchAll() async throws {
        let mock = MockTagPort()
        let tags = [
            Frank.Tag(id: UUID(), name: "AI", category: "ai"),
            Frank.Tag(id: UUID(), name: "iOS", category: "ios"),
        ]
        mock.allTags = tags

        let result = try await mock.fetchAllTags()

        #expect(result == tags)
        #expect(mock.fetchAllTagsCallCount == 1)
    }

    @Test("MockTagPort 내 태그 ID 반환")
    func tagFetchMyIds() async throws {
        let mock = MockTagPort()
        let ids = [UUID(), UUID()]
        mock.myTagIds = ids

        let result = try await mock.fetchMyTagIds()

        #expect(result == ids)
    }

    @Test("MockTagPort 태그 저장 추적")
    func tagSave() async throws {
        let mock = MockTagPort()
        let ids = [UUID(), UUID()]

        try await mock.saveMyTags(tagIds: ids)

        #expect(mock.saveMyTagsCallCount == 1)
        #expect(mock.savedTagIds == ids)
    }

    // MARK: - ArticlePort

    @Test("MockArticlePort 기사 목록 limit 적용")
    func articleFetchWithLimit() async throws {
        let mock = MockArticlePort()
        let tagId = UUID()
        mock.articles = (0..<5).map { i in
            Article(
                id: UUID(),
                title: "Article \(i)",
                url: URL(string: "https://example.com/\(i)")!,
                source: "Test",
                publishedAt: Date(),
                summary: nil,
                tagId: tagId
            )
        }

        let result = try await mock.fetchArticles(limit: 3)

        #expect(result.count == 3)
        #expect(mock.fetchArticlesCallCount == 1)
    }

    @Test("MockArticlePort 기사 상세 조회")
    func articleFetchDetail() async throws {
        let mock = MockArticlePort()
        let articleId = UUID()
        let article = Article(
            id: articleId,
            title: "Test",
            url: URL(string: "https://example.com")!,
            source: "Test",
            publishedAt: Date(),
            summary: "요약",
            tagId: UUID()
        )
        mock.articles = [article]

        let result = try await mock.fetchArticle(id: articleId)

        #expect(result == article)
    }

    @Test("MockArticlePort 조회 실패")
    func articleFetchError() async {
        let mock = MockArticlePort()
        mock.fetchError = URLError(.timedOut)

        await #expect(throws: URLError.self) {
            try await mock.fetchArticles(limit: 10)
        }
    }

    // MARK: - CollectPort

    @Test("MockCollectPort collect 호출 추적")
    func collectTrigger() async throws {
        let mock = MockCollectPort()

        try await mock.triggerCollect()

        #expect(mock.triggerCollectCallCount == 1)
    }

    @Test("MockCollectPort summarize 에러 전파")
    func summarizeError() async {
        let mock = MockCollectPort()
        mock.summarizeError = URLError(.badServerResponse)

        await #expect(throws: URLError.self) {
            try await mock.triggerSummarize()
        }
        #expect(mock.triggerSummarizeCallCount == 1)
    }
}
