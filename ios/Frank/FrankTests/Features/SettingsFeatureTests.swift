import Testing
import Foundation
@testable import Frank

@MainActor
@Suite("SettingsFeature Tests")
struct SettingsFeatureTests {

    // MARK: - Helpers

    private func makeSUT(
        allTags: [Frank.Tag] = [],
        myTagIds: [UUID] = [],
        fetchError: Error? = nil,
        saveError: Error? = nil,
        signOutError: Error? = nil
    ) -> (SettingsFeature, MockTagPort, MockAuthPort) {
        let tagPort = MockTagPort()
        tagPort.allTags = allTags
        tagPort.myTagIds = myTagIds
        tagPort.fetchError = fetchError
        tagPort.saveError = saveError

        let authPort = MockAuthPort()
        authPort.signOutError = signOutError

        let feature = SettingsFeature(tag: tagPort, auth: authPort)
        return (feature, tagPort, authPort)
    }

    private func sampleTags() -> [Frank.Tag] {
        [
            Frank.Tag(id: UUID(), name: "Swift", category: "tech"),
            Frank.Tag(id: UUID(), name: "Rust", category: "tech"),
            Frank.Tag(id: UUID(), name: "AI", category: "tech"),
        ]
    }

    // MARK: - loadTags

    @Test("loadTags: 전체 태그와 내 태그를 로드한다")
    func loadTags_success() async {
        let tags = sampleTags()
        let myIds = [tags[0].id, tags[2].id]
        let (sut, tagPort, _) = makeSUT(allTags: tags, myTagIds: myIds)

        await sut.send(.loadTags)

        #expect(sut.tags == tags)
        #expect(sut.selectedIds == Set(myIds))
        #expect(sut.originalIds == Set(myIds))
        #expect(tagPort.fetchAllTagsCallCount == 1)
        #expect(tagPort.fetchMyTagIdsCallCount == 1)
    }

    @Test("loadTags: 실패 시 에러 메시지를 표시한다")
    func loadTags_failure() async {
        let (sut, _, _) = makeSUT(fetchError: URLError(.notConnectedToInternet))

        await sut.send(.loadTags)

        #expect(sut.tags.isEmpty)
        #expect(sut.errorMessage != nil)
    }

    // MARK: - toggleTag

    @Test("toggleTag: 선택되지 않은 태그를 선택한다")
    func toggleTag_select() async {
        let tags = sampleTags()
        let (sut, _, _) = makeSUT(allTags: tags, myTagIds: [tags[0].id])

        await sut.send(.loadTags)
        await sut.send(.toggleTag(tags[1].id))

        #expect(sut.selectedIds.contains(tags[1].id))
    }

    @Test("toggleTag: 선택된 태그를 해제한다")
    func toggleTag_deselect() async {
        let tags = sampleTags()
        let (sut, _, _) = makeSUT(allTags: tags, myTagIds: [tags[0].id, tags[1].id])

        await sut.send(.loadTags)
        await sut.send(.toggleTag(tags[0].id))

        #expect(!sut.selectedIds.contains(tags[0].id))
    }

    @Test("toggleTag: 마지막 1개는 해제할 수 없다")
    func toggleTag_cannotDeselectLast() async {
        let tags = sampleTags()
        let (sut, _, _) = makeSUT(allTags: tags, myTagIds: [tags[0].id])

        await sut.send(.loadTags)
        await sut.send(.toggleTag(tags[0].id))

        #expect(sut.selectedIds.contains(tags[0].id))
    }

    // MARK: - saveTags

    @Test("saveTags: 변경된 태그를 저장한다")
    func saveTags_success() async {
        let tags = sampleTags()
        let (sut, tagPort, _) = makeSUT(allTags: tags, myTagIds: [tags[0].id])

        await sut.send(.loadTags)
        await sut.send(.toggleTag(tags[1].id))
        await sut.send(.saveTags)

        #expect(tagPort.saveMyTagsCallCount == 1)
        #expect(Set(tagPort.savedTagIds ?? []) == Set([tags[0].id, tags[1].id]))
        #expect(sut.tagsChanged)
    }

    @Test("saveTags: 변경이 없으면 저장하지 않는다")
    func saveTags_noChange() async {
        let tags = sampleTags()
        let (sut, tagPort, _) = makeSUT(allTags: tags, myTagIds: [tags[0].id])

        await sut.send(.loadTags)
        await sut.send(.saveTags)

        #expect(tagPort.saveMyTagsCallCount == 0)
        #expect(!sut.tagsChanged)
    }

    @Test("saveTags: 실패 시 에러 메시지를 표시한다")
    func saveTags_failure() async {
        let tags = sampleTags()
        let (sut, _, _) = makeSUT(
            allTags: tags,
            myTagIds: [tags[0].id],
            saveError: URLError(.networkConnectionLost)
        )

        await sut.send(.loadTags)
        await sut.send(.toggleTag(tags[1].id))
        await sut.send(.saveTags)

        #expect(sut.errorMessage != nil)
        #expect(!sut.tagsChanged)
    }

    @Test("saveTags: 저장 후 originalIds가 갱신된다")
    func saveTags_updatesOriginalIds() async {
        let tags = sampleTags()
        let (sut, _, _) = makeSUT(allTags: tags, myTagIds: [tags[0].id])

        await sut.send(.loadTags)
        await sut.send(.toggleTag(tags[1].id))
        await sut.send(.saveTags)

        #expect(sut.originalIds == sut.selectedIds)
    }

    // MARK: - canSave

    @Test("canSave: 변경이 있고 비어있지 않으면 true")
    func canSave_true() async {
        let tags = sampleTags()
        let (sut, _, _) = makeSUT(allTags: tags, myTagIds: [tags[0].id])

        await sut.send(.loadTags)
        await sut.send(.toggleTag(tags[1].id))

        #expect(sut.canSave)
    }

    @Test("canSave: 변경이 없으면 false")
    func canSave_noChange() async {
        let tags = sampleTags()
        let (sut, _, _) = makeSUT(allTags: tags, myTagIds: [tags[0].id])

        await sut.send(.loadTags)

        #expect(!sut.canSave)
    }

    // MARK: - signOut

    @Test("signOut: AuthPort.signOut을 호출한다")
    func signOut_success() async {
        let (sut, _, authPort) = makeSUT()

        await sut.send(.signOut)

        #expect(authPort.signOutCallCount == 1)
    }

    @Test("signOut: 실패 시 에러 메시지를 표시한다")
    func signOut_failure() async {
        let (sut, _, _) = makeSUT(signOutError: URLError(.networkConnectionLost))

        await sut.send(.signOut)

        #expect(sut.errorMessage != nil)
    }

    // MARK: - isLoading

    @Test("loadTags 중 isLoading이 true이다")
    func loadTags_isLoading() async {
        let tags = sampleTags()
        let (sut, _, _) = makeSUT(allTags: tags, myTagIds: [])

        #expect(!sut.isLoading)
        await sut.send(.loadTags)
        #expect(!sut.isLoading) // 완료 후 false
    }

    // MARK: - MVP15 M3: Occupation

    @Test("loadOccupation: 현재 프로필에서 occupation을 로드한다")
    func loadOccupation_success() async {
        let (sut, _, authPort) = makeSUT()
        authPort.currentProfileResult = Profile(
            id: UUID(),
            displayName: "test",
            onboardingCompleted: true,
            occupation: "iOS 개발자"
        )

        await sut.send(.loadOccupation)

        #expect(sut.occupation == "iOS 개발자")
        #expect(authPort.currentProfileCallCount == 1)
    }

    @Test("loadOccupation: occupation 미설정 시 nil")
    func loadOccupation_nil() async {
        let (sut, _, authPort) = makeSUT()
        authPort.currentProfileResult = Profile(
            id: UUID(),
            displayName: "test",
            onboardingCompleted: true,
            occupation: nil
        )

        await sut.send(.loadOccupation)

        #expect(sut.occupation == nil)
    }

    @Test("saveOccupation: 직업을 저장하고 occupation을 갱신한다")
    func saveOccupation_success() async {
        let (sut, _, authPort) = makeSUT()
        authPort.updateOccupationResult = .success(
            Profile(id: UUID(), displayName: "test", onboardingCompleted: true, occupation: "프론트엔드 개발자")
        )

        await sut.send(.saveOccupation("프론트엔드 개발자"))

        #expect(sut.occupation == "프론트엔드 개발자")
        #expect(sut.occupationSuccess != nil)
        #expect(sut.occupationError == nil)
        #expect(authPort.updateOccupationCallCount == 1)
        #expect(authPort.lastUpdatedOccupation!! == "프론트엔드 개발자")
    }

    @Test("saveOccupation: 빈 문자열은 nil로 처리된다")
    func saveOccupation_emptyStringToNil() async {
        let (sut, _, authPort) = makeSUT()
        authPort.updateOccupationResult = .success(
            Profile(id: UUID(), displayName: "test", onboardingCompleted: true, occupation: nil)
        )

        await sut.send(.saveOccupation(""))

        #expect(sut.occupation == nil)
        // 빈 문자열은 nil로 정규화되어야 함
        #expect(authPort.lastUpdatedOccupation! == nil)
    }

    @Test("saveOccupation: nil 전달 시 직업 삭제")
    func saveOccupation_delete() async {
        let (sut, _, authPort) = makeSUT()
        authPort.updateOccupationResult = .success(
            Profile(id: UUID(), displayName: "test", onboardingCompleted: true, occupation: nil)
        )

        await sut.send(.saveOccupation(nil))

        #expect(sut.occupation == nil)
        #expect(authPort.lastUpdatedOccupation! == nil)
    }

    @Test("saveOccupation: 실패 시 에러 메시지를 표시한다")
    func saveOccupation_failure() async {
        let (sut, _, authPort) = makeSUT()
        authPort.updateOccupationResult = .failure(URLError(.networkConnectionLost))

        await sut.send(.saveOccupation("개발자"))

        #expect(sut.occupationError != nil)
        #expect(sut.occupationSuccess == nil)
    }

    @Test("saveOccupation: 50자 초과 시 에러 메시지를 표시하고 API를 호출하지 않는다")
    func saveOccupation_tooLong() async {
        let (sut, _, authPort) = makeSUT()
        let tooLong = String(repeating: "가", count: 51)

        await sut.send(.saveOccupation(tooLong))

        #expect(sut.occupationError != nil)
        #expect(sut.occupationSuccess == nil)
        #expect(authPort.updateOccupationCallCount == 0)
    }

    @Test("saveOccupation: 50자 초과 시 이전 성공 메시지도 지워진다")
    func saveOccupation_tooLong_clearsSuccessMessage() async {
        let (sut, _, authPort) = makeSUT()
        authPort.updateOccupationResult = .success(
            Profile(id: UUID(), displayName: "test", onboardingCompleted: true, occupation: "개발자")
        )
        // 먼저 성공 저장
        await sut.send(.saveOccupation("개발자"))
        #expect(sut.occupationSuccess != nil)

        // 이후 50자 초과 입력 — 성공 메시지도 지워져야 함
        let tooLong = String(repeating: "가", count: 51)
        await sut.send(.saveOccupation(tooLong))

        #expect(sut.occupationError != nil)
        #expect(sut.occupationSuccess == nil)
    }

    @Test("saveOccupation: 개행 포함 공백은 nil로 처리된다")
    func saveOccupation_newlineOnlyIsNil() async {
        let (sut, _, authPort) = makeSUT()
        authPort.updateOccupationResult = .success(
            Profile(id: UUID(), displayName: "test", onboardingCompleted: true, occupation: nil)
        )

        await sut.send(.saveOccupation("\n"))

        // 개행만 있는 입력은 nil 처리 (삭제)
        #expect(authPort.lastUpdatedOccupation! == nil)
    }

    @Test("saveOccupation: 공백 포함 유효 직업명은 trim 후 저장된다")
    func saveOccupation_trimmedValue() async {
        let (sut, _, authPort) = makeSUT()
        authPort.updateOccupationResult = .success(
            Profile(id: UUID(), displayName: "test", onboardingCompleted: true, occupation: "iOS 개발자")
        )

        await sut.send(.saveOccupation("  iOS 개발자  "))

        // trim 후 "iOS 개발자"로 저장 (서버에도 trim된 값 전달)
        #expect(authPort.lastUpdatedOccupation!! == "iOS 개발자")
    }
}
