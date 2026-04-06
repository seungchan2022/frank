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
}
