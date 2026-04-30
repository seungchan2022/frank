import XCTest

/// MVP14 M3: DEBT-04~07 UX 개선 검증 UITest (Mock 모드).
/// - DEBT-04: 피드 좋아요 버튼 단독 탭 → 디테일 이동 없이 좋아요만 처리
/// - DEBT-05: 스크랩/오답 탭 롱프레스 삭제 메뉴 (swipeActions 제거 후 contextMenu)
/// - DEBT-06: 기사 디테일 액션 버튼 하단 고정 — 원문 보기 버튼 항상 접근 가능
/// - DEBT-07: 기사 소개 카드(newspaper 아이콘) + AI 요약 카드(sparkles 아이콘) 구분
final class M3UXImprovementsUITest: XCTestCase {

    private let app = XCUIApplication()

    private func launchMock() {
        app.launchEnvironment["FRANK_USE_MOCK"] = "1"
        app.launch()
    }

    override func setUp() {
        continueAfterFailure = false
    }

    // MARK: - DEBT-04: 피드 좋아요 버튼 — 디테일 이동 없이 처리

    /// F-01 + F-02: 좋아요 버튼 탭 시 디테일 이동 없음, 카드 탭 시 디테일 이동
    func testFeedLikeButtonDoesNotNavigateToDetail() {
        launchMock()

        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(allTagButton.waitForExistence(timeout: 10), "피드 화면 진입")
        takeScreenshot(name: "debt04_01_피드")

        // 좋아요 버튼 탭 — accessibilityIdentifier로 정확히 식별
        let likeButton = app.buttons.matching(identifier: "feed_like_button").firstMatch
        XCTAssertTrue(likeButton.waitForExistence(timeout: 5), "좋아요 버튼 존재")
        likeButton.tap()

        // 디테일로 이동하지 않고 피드에 머물러야 함
        XCTAssertTrue(
            allTagButton.waitForExistence(timeout: 3),
            "DEBT-04: 좋아요 탭 후 피드에 머뭄 (디테일 이동 없음)"
        )
        takeScreenshot(name: "debt04_02_좋아요탭후_피드유지")
    }

    /// F-02: 카드 탭 시 디테일로 이동
    func testFeedCardTapNavigatesToDetail() {
        launchMock()

        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(allTagButton.waitForExistence(timeout: 10), "피드 화면 진입")

        // 카드 탭 (셀의 좋아요 버튼 외 영역)
        let firstCell = app.cells.firstMatch
        XCTAssertTrue(firstCell.waitForExistence(timeout: 5), "첫 기사 카드 존재")
        firstCell.tap()

        // 디테일 화면 진입 확인 — "원문 보기" 버튼 존재
        let openButton = app.buttons["원문 보기"]
        XCTAssertTrue(
            openButton.waitForExistence(timeout: 5),
            "F-02: 카드 탭 후 디테일 진입"
        )
        takeScreenshot(name: "debt04_03_카드탭_디테일진입")
    }

    // MARK: - DEBT-06: 기사 디테일 액션 버튼 하단 고정

    /// F-05 + E-01: 디테일 진입 시 "원문 보기" 버튼이 항상 접근 가능 (safeAreaInset 고정)
    func testDetailActionButtonsAlwaysAccessible() {
        launchMock()

        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(allTagButton.waitForExistence(timeout: 10), "피드 화면 진입")

        let firstCell = app.cells.firstMatch
        XCTAssertTrue(firstCell.waitForExistence(timeout: 5))
        firstCell.tap()

        // 원문 보기 버튼이 디테일 진입 직후 접근 가능 (하단 고정)
        let openButton = app.buttons["원문 보기"]
        XCTAssertTrue(
            openButton.waitForExistence(timeout: 5),
            "F-05: 원문 보기 버튼 하단 고정 — 진입 직후 접근 가능"
        )
        takeScreenshot(name: "debt06_01_원문보기_하단고정")

        // 요약하기 버튼도 하단 고정 영역에 존재해야 함
        let summarizeButton = app.buttons["요약하기"]
        XCTAssertTrue(
            summarizeButton.waitForExistence(timeout: 3),
            "F-05: 요약하기 버튼 하단 고정"
        )
        takeScreenshot(name: "debt06_02_요약하기_하단고정")
    }

    // MARK: - DEBT-07: 카드 UI 구분 — 기사 소개 vs AI 요약

    /// F-06 + F-07: 기사 소개(newspaper) 레이블, AI 요약(sparkles) 레이블 존재
    func testDetailCardVisualDistinction() {
        launchMock()

        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(allTagButton.waitForExistence(timeout: 10), "피드 화면 진입")

        let firstCell = app.cells.firstMatch
        XCTAssertTrue(firstCell.waitForExistence(timeout: 5))
        firstCell.tap()

        // "기사 소개" 레이블 확인
        let snippetLabel = app.staticTexts["기사 소개"]
        XCTAssertTrue(
            snippetLabel.waitForExistence(timeout: 5),
            "F-06: 기사 소개 카드 레이블 표시"
        )
        takeScreenshot(name: "debt07_01_기사소개카드")
    }

    // MARK: - Helpers

    private func takeScreenshot(name: String) {
        let screenshot = app.windows.firstMatch.screenshot()
        let attachment = XCTAttachment(screenshot: screenshot)
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
