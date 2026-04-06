import Testing
import Foundation
@testable import Frank

@Suite("OnboardingFeature Tests")
@MainActor
struct OnboardingFeatureTests {

    private func makeSUT(
        tags: [Frank.Tag] = [],
        myTagIds: [UUID] = [],
        saveError: Error? = nil,
        fetchError: Error? = nil,
        updateOnboardingResult: Result<Profile, Error>? = nil
    ) -> (OnboardingFeature, MockTagPort, MockAuthPort, CompletionTracker) {
        let tagPort = MockTagPort()
        tagPort.allTags = tags
        tagPort.myTagIds = myTagIds
        tagPort.saveError = saveError
        tagPort.fetchError = fetchError

        let authPort = MockAuthPort()
        if let result = updateOnboardingResult {
            authPort.updateOnboardingCompletedResult = result
        }

        let tracker = CompletionTracker()
        let feature = OnboardingFeature(
            tag: tagPort,
            auth: authPort,
            onComplete: { tracker.called = true }
        )
        return (feature, tagPort, authPort, tracker)
    }

    // MARK: - loadTags

    @Test("초기 상태는 loading")
    func initialState() {
        let (sut, _, _, _) = makeSUT()
        #expect(sut.state == .loading)
        #expect(sut.isSaving == false)
        #expect(sut.errorMessage == nil)
    }

    @Test("loadTags 성공 시 loaded 상태로 전환")
    func loadTagsSuccess() async {
        let tags = [
            Frank.Tag(id: UUID(), name: "AI", slug: "ai"),
            Frank.Tag(id: UUID(), name: "iOS", slug: "ios"),
        ]
        let (sut, tagPort, _, _) = makeSUT(tags: tags)

        await sut.send(.loadTags)

        if case let .loaded(loadedTags, selectedIds) = sut.state {
            #expect(loadedTags == tags)
            #expect(selectedIds.isEmpty)
        } else {
            Issue.record("Expected loaded state, got \(sut.state)")
        }
        #expect(tagPort.fetchAllTagsCallCount == 1)
    }

    @Test("loadTags 시 기존 선택된 태그 복원")
    func loadTagsRestoresSelection() async {
        let tagId1 = UUID()
        let tagId2 = UUID()
        let tags = [
            Frank.Tag(id: tagId1, name: "AI", slug: "ai"),
            Frank.Tag(id: tagId2, name: "iOS", slug: "ios"),
        ]
        let (sut, _, _, _) = makeSUT(tags: tags, myTagIds: [tagId1])

        await sut.send(.loadTags)

        if case let .loaded(_, selectedIds) = sut.state {
            #expect(selectedIds == Set([tagId1]))
        } else {
            Issue.record("Expected loaded state")
        }
    }

    @Test("loadTags 실패 시 errorMessage 설정 + 빈 loaded 상태")
    func loadTagsFailure() async {
        let (sut, _, _, _) = makeSUT(fetchError: URLError(.notConnectedToInternet))

        await sut.send(.loadTags)

        if case let .loaded(tags, _) = sut.state {
            #expect(tags.isEmpty)
        } else {
            Issue.record("Expected loaded state with empty tags")
        }
        #expect(sut.errorMessage != nil)
    }

    // MARK: - toggleTag

    @Test("태그 선택 토글 — 선택/해제")
    func toggleTag() async {
        let tagId = UUID()
        let tags = [Frank.Tag(id: tagId, name: "AI", slug: "ai")]
        let (sut, _, _, _) = makeSUT(tags: tags)

        await sut.send(.loadTags)
        await sut.send(.toggleTag(id: tagId))

        if case let .loaded(_, selectedIds) = sut.state {
            #expect(selectedIds.contains(tagId))
        } else {
            Issue.record("Expected loaded state")
        }

        await sut.send(.toggleTag(id: tagId))

        if case let .loaded(_, selectedIds) = sut.state {
            #expect(!selectedIds.contains(tagId))
        } else {
            Issue.record("Expected loaded state")
        }
    }

    // MARK: - canComplete

    @Test("선택 없으면 complete 불가")
    func cannotCompleteWithNoSelection() async {
        let tags = [Frank.Tag(id: UUID(), name: "AI", slug: "ai")]
        let (sut, _, _, _) = makeSUT(tags: tags)

        await sut.send(.loadTags)

        #expect(sut.canComplete == false)
    }

    @Test("1개 이상 선택 시 complete 가능")
    func canCompleteWithSelection() async {
        let tagId = UUID()
        let tags = [Frank.Tag(id: tagId, name: "AI", slug: "ai")]
        let (sut, _, _, _) = makeSUT(tags: tags)

        await sut.send(.loadTags)
        await sut.send(.toggleTag(id: tagId))

        #expect(sut.canComplete == true)
    }

    // MARK: - complete

    @Test("complete 성공 시 saveMyTags + updateOnboarding + onComplete 호출")
    func completeSuccess() async {
        let tagId = UUID()
        let tags = [Frank.Tag(id: tagId, name: "AI", slug: "ai")]
        let updatedProfile = Profile(id: UUID(), email: "test@example.com", onboardingCompleted: true)
        let (sut, tagPort, authPort, tracker) = makeSUT(
            tags: tags,
            updateOnboardingResult: .success(updatedProfile)
        )

        await sut.send(.loadTags)
        await sut.send(.toggleTag(id: tagId))
        await sut.send(.complete)

        #expect(tagPort.saveMyTagsCallCount == 1)
        #expect(tagPort.savedTagIds == [tagId])
        #expect(authPort.updateOnboardingCompletedCallCount == 1)
        #expect(tracker.called == true)
        #expect(sut.isSaving == false)
    }

    @Test("saveMyTags 실패 시 errorMessage + 태그 선택 유지")
    func completeSaveFailure() async {
        let tagId = UUID()
        let tags = [Frank.Tag(id: tagId, name: "AI", slug: "ai")]
        let (sut, _, _, tracker) = makeSUT(tags: tags, saveError: URLError(.timedOut))

        await sut.send(.loadTags)
        await sut.send(.toggleTag(id: tagId))
        await sut.send(.complete)

        // 에러 메시지 표시
        #expect(sut.errorMessage != nil)
        #expect(sut.isSaving == false)
        // 태그 선택 유지 확인
        if case let .loaded(_, selectedIds) = sut.state {
            #expect(selectedIds.contains(tagId))
        } else {
            Issue.record("Expected loaded state with selections preserved")
        }
        // onComplete 호출되지 않음
        #expect(tracker.called == false)
    }

    @Test("updateOnboardingCompleted 실패 시 errorMessage + 태그 선택 유지")
    func completeUpdateProfileFailure() async {
        let tagId = UUID()
        let tags = [Frank.Tag(id: tagId, name: "AI", slug: "ai")]
        let (sut, _, _, tracker) = makeSUT(
            tags: tags,
            updateOnboardingResult: .failure(URLError(.badServerResponse))
        )

        await sut.send(.loadTags)
        await sut.send(.toggleTag(id: tagId))
        await sut.send(.complete)

        #expect(sut.errorMessage != nil)
        #expect(sut.isSaving == false)
        if case let .loaded(_, selectedIds) = sut.state {
            #expect(selectedIds.contains(tagId))
        } else {
            Issue.record("Expected loaded state with selections preserved")
        }
        #expect(tracker.called == false)
    }

    @Test("선택 없이 complete 호출 시 무시")
    func completeWithNoSelectionIgnored() async {
        let tags = [Frank.Tag(id: UUID(), name: "AI", slug: "ai")]
        let (sut, tagPort, authPort, _) = makeSUT(tags: tags)

        await sut.send(.loadTags)
        await sut.send(.complete)

        #expect(tagPort.saveMyTagsCallCount == 0)
        #expect(authPort.updateOnboardingCompletedCallCount == 0)
    }

    // MARK: - clearError

    @Test("clearError로 에러 메시지 초기화")
    func clearError() async {
        let (sut, _, _, _) = makeSUT(fetchError: URLError(.notConnectedToInternet))

        await sut.send(.loadTags)
        #expect(sut.errorMessage != nil)

        sut.clearError()
        #expect(sut.errorMessage == nil)
    }

    // MARK: - retry

    @Test("에러 후 retry 시 다시 loadTags 성공")
    func retryAfterError() async {
        let tags = [Frank.Tag(id: UUID(), name: "AI", slug: "ai")]
        let tagPort = MockTagPort()
        tagPort.fetchError = URLError(.notConnectedToInternet)
        let authPort = MockAuthPort()
        let sut = OnboardingFeature(tag: tagPort, auth: authPort, onComplete: {})

        await sut.send(.loadTags)
        #expect(sut.errorMessage != nil)

        tagPort.fetchError = nil
        tagPort.allTags = tags
        await sut.send(.loadTags)

        if case let .loaded(loadedTags, _) = sut.state {
            #expect(loadedTags == tags)
        } else {
            Issue.record("Expected loaded state")
        }
        #expect(sut.errorMessage == nil)
    }
}

// MARK: - Test Helpers

@MainActor
final class CompletionTracker {
    var called = false
}
