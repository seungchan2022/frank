import XCTest

/// MVP16 M4 E2E 검증 — E1/F2/D2
///
/// E1: 피드 카드 태그 칩 표시
/// F2: 오답노트 태그 필터 동작
/// D2: 오답 원문 보기 버튼 표시 + 빈 URL 비활성
final class M4VerificationUITest: XCTestCase {

    private let app = XCUIApplication()

    override func setUp() {
        continueAfterFailure = false
    }

    // MARK: - E1: 피드 카드 태그 칩 표시

    func testFeedCardTagChipVisible() {
        app.launchMock()

        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(allTagButton.waitForExistence(timeout: 10), "피드 화면 진입")
        takeScreenshot(name: "E1_01_피드화면")

        // Mock fixture 기사 중 태그 있는 항목: AI/ML, iOS 개발 등
        // staticTexts로 태그 칩 텍스트 탐색 (accessibilityElement combine 하위 — HStack 내 캡션 레이블)
        let tagChipAIML = app.staticTexts["AI/ML"]
        XCTAssertTrue(tagChipAIML.waitForExistence(timeout: 5), "E1: 피드 카드에 AI/ML 태그 칩 표시")
        takeScreenshot(name: "E1_02_태그칩_확인")
    }

    // MARK: - F2 + D2: 오답노트 (시드 데이터 포함)

    func testWrongAnswersTabFilterAndArticleURL() {
        app.launchEnvironment["FRANK_USE_MOCK"] = "1"
        app.launchEnvironment["FRANK_UI_SCENARIO"] = "with_wrong_answers"
        app.launch()

        // 피드 진입 확인
        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(allTagButton.waitForExistence(timeout: 10), "피드 화면 진입")

        // 스크랩 탭 이동
        let scrapTab = app.tabBars.buttons["스크랩"]
        XCTAssertTrue(scrapTab.waitForExistence(timeout: 5), "스크랩 탭 존재")
        scrapTab.tap()
        sleep(2)
        takeScreenshot(name: "F2_01_스크랩탭")

        // "오답 노트" 세그먼트 선택
        let wrongAnswerTab = app.buttons["오답 노트"]
        XCTAssertTrue(wrongAnswerTab.waitForExistence(timeout: 5), "'오답 노트' 세그먼트 존재")
        wrongAnswerTab.tap()
        sleep(3)
        takeScreenshot(name: "F2_02_오답노트탭")

        // F2: 태그 필터 칩 확인 (AI/ML)
        // TagChipView → Button { Text(tag.name) } → label = tag.name
        let aimlFilterChip = app.buttons["AI/ML"]
        XCTAssertTrue(
            aimlFilterChip.waitForExistence(timeout: 8),
            "F2: 오답노트 AI/ML 태그 필터 칩 존재"
        )
        takeScreenshot(name: "F2_03_태그필터칩_확인")

        // F2: AI/ML 칩 탭 → 필터 적용
        aimlFilterChip.tap()
        sleep(2)
        takeScreenshot(name: "F2_04_태그필터_AIML_적용")

        // F2: AI/ML 칩 선택 상태 확인 (TagChipView.accessibilityAddTraits(.isSelected))
        XCTAssertTrue(aimlFilterChip.isSelected, "F2: AI/ML 칩이 선택됨")

        // F2: 전체 칩은 선택 해제 상태
        let allChipDuringFilter = app.buttons["전체"].firstMatch
        XCTAssertFalse(allChipDuringFilter.isSelected, "F2: 전체 칩이 선택 해제됨")

        takeScreenshot(name: "F2_05_AIML_칩_선택상태_확인")

        // 필터 해제 (전체로 복귀)
        allChipDuringFilter.tap()
        sleep(1)

        // D2: 원문 보기 버튼 — articleUrl 있는 항목에만 표시
        let articleURLButton = app.buttons["원문 기사 열기"]
        XCTAssertTrue(
            articleURLButton.waitForExistence(timeout: 5),
            "D2: 원문 URL 있는 오답에 '원문 기사 열기' 버튼 표시"
        )
        takeScreenshot(name: "D2_01_원문보기버튼")
    }

    // MARK: - Helpers

    private func takeScreenshot(name: String) {
        takeScreenshot(app: app, name: name)
    }
}
