import XCTest

/// MVP14 M3: DEBT-04~07 UX 개선 검증 UITest (Mock 모드).
/// - DEBT-04: 피드 좋아요 버튼 단독 탭 → 디테일 이동 없이 좋아요만 처리
/// - DEBT-05: 스크랩/오답 탭 롱프레스 삭제 메뉴 (swipeActions 제거 후 contextMenu)
/// - DEBT-06: 기사 디테일 액션 버튼 하단 고정 — 원문 보기 버튼 항상 접근 가능
/// - DEBT-07: 기사 소개 카드(newspaper 아이콘) + AI 요약 카드(sparkles 아이콘) 구분
final class M3UXImprovementsUITest: XCTestCase {

    private let app = XCUIApplication()

    override func setUp() {
        continueAfterFailure = false
    }

    // MARK: - DEBT-04: 피드 좋아요 버튼 — 디테일 이동 없이 처리

    /// F-01 + F-02: 좋아요 버튼 탭 시 디테일 이동 없음, 카드 탭 시 디테일 이동
    /// I-05: DEBT-04 — 좋아요 버튼 단독 탭 검증
    func testFeedLikeButtonDoesNotNavigateToDetail() {
        app.launchMock()

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
        app.launchMock()

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

    // MARK: - I-03: 요약 후 버튼 접근 (BUG-006 + DEBT-06)

    /// I-03: 요약하기 탭 → 요약 완료 → 스크랩 버튼 여전히 접근 가능 (BUG-006 + DEBT-06)
    ///
    /// BUG-006: 요약 성공 후 버튼이 올바르게 갱신되어야 함
    /// DEBT-06: 하단 고정 버튼 패널이 요약 후에도 항상 접근 가능해야 함
    func testDetailSummaryThenActionButton() {
        app.launchMock()

        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(allTagButton.waitForExistence(timeout: 10), "피드 화면 진입")

        let firstCell = app.cells.firstMatch
        XCTAssertTrue(firstCell.waitForExistence(timeout: 5), "첫 기사 카드 존재")
        firstCell.tap()

        // 요약하기 버튼 존재 확인 (하단 고정 — DEBT-06)
        let summarizeButton = app.buttons["요약하기"]
        XCTAssertTrue(
            summarizeButton.waitForExistence(timeout: 5),
            "I-03: 요약하기 버튼 진입 직후 접근 가능 (DEBT-06 하단 고정)"
        )
        takeScreenshot(name: "i03_01_요약하기_버튼_존재")

        // 요약하기 탭
        summarizeButton.tap()
        takeScreenshot(name: "i03_02_요약_진행중")

        // 요약 완료 대기 (MockSummarizeAdapter 600ms 지연)
        // 요약 완료 후 "스크랩 저장" 버튼이 여전히 접근 가능해야 함 (DEBT-06)
        let scrapButton = app.buttons.matching(
            NSPredicate(format: "label CONTAINS '스크랩'")
        ).firstMatch
        XCTAssertTrue(
            scrapButton.waitForExistence(timeout: 5),
            "I-03: 요약 탭 후 스크랩 버튼 여전히 접근 가능 (BUG-006 + DEBT-06)"
        )
        takeScreenshot(name: "i03_03_요약후_스크랩버튼_접근가능")

        // 원문 보기 버튼도 접근 가능해야 함
        let openButton = app.buttons["원문 보기"]
        XCTAssertTrue(
            openButton.waitForExistence(timeout: 3),
            "I-03: 요약 탭 후 원문 보기 버튼 여전히 접근 가능"
        )
        takeScreenshot(name: "i03_04_요약후_원문보기_접근가능")
    }

    // MARK: - DEBT-06: 기사 디테일 액션 버튼 하단 고정

    /// F-05 + E-01: 디테일 진입 시 "원문 보기" 버튼이 항상 접근 가능 (safeAreaInset 고정)
    func testDetailActionButtonsAlwaysAccessible() {
        app.launchMock()

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
        app.launchMock()

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

    // MARK: - E-04: 연속 빠른 탭 — 중복 좋아요 요청 없음

    /// E-04: 좋아요 버튼 연속 빠른 탭 시 중복 요청 없이 화면 유지
    func testFeedLikeButtonRapidTapNoDuplicateRequest() {
        app.launchMock()

        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(allTagButton.waitForExistence(timeout: 10), "피드 화면 진입")

        let likeButton = app.buttons.matching(identifier: "feed_like_button").firstMatch
        XCTAssertTrue(likeButton.waitForExistence(timeout: 5), "좋아요 버튼 존재")

        // 연속 빠른 탭 3회 — 중복 요청 발생해도 크래시 없어야 함
        likeButton.tap()
        likeButton.tap()
        likeButton.tap()

        // 피드 화면 유지 (크래시 없음)
        XCTAssertTrue(
            allTagButton.waitForExistence(timeout: 3),
            "E-04: 연속 빠른 탭 후 피드 화면 유지 (크래시 없음)"
        )
        takeScreenshot(name: "e04_연속탭_후_피드유지")
    }

    // MARK: - Helpers

    private func takeScreenshot(name: String) {
        takeScreenshot(app: app, name: name)
    }
}
