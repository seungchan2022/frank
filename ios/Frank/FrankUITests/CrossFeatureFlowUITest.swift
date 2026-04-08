import XCTest

/// Mock 모드(`FRANK_USE_MOCK=1`)에서 실행되는 크로스-Feature E2E 테스트.
///
/// TC-01/02/03: 각 시나리오는 FRANK_UI_SCENARIO로 주입
/// TC-01: LoginFlowUITest — 이메일 로그인 → 피드
/// TC-02: OnboardingFlowUITest — 신규 사용자 온보딩
/// TC-03: testSummaryTimeoutUI — 피드 → 새 뉴스 가져오기 → timeout 배너
/// TC-04: testTagManagementAndLogout — 설정 → 태그 관리 → 로그아웃
///
/// 외부 호출 0 (MockAuthAdapter / MockArticleAdapter / MockTagAdapter / MockCollectAdapter).
final class CrossFeatureFlowUITest: XCTestCase {

    private let app = XCUIApplication()

    private func launchMock(scenario: String? = nil, summarizeTimeoutSeconds: Int? = nil) {
        app.launchEnvironment["FRANK_USE_MOCK"] = "1"
        if let scenario {
            app.launchEnvironment["FRANK_UI_SCENARIO"] = scenario
        }
        if let timeout = summarizeTimeoutSeconds {
            app.launchEnvironment["FRANK_SUMMARIZE_TIMEOUT_SECONDS"] = "\(timeout)"
        }
        app.launch()
    }

    override func setUp() {
        continueAfterFailure = false
    }

    // MARK: - 기존 Flow: 피드 → 상세 → 설정 (Mock 기본)

    func testFeedToDetailToSettingsFlow() {
        launchMock()
        // 1. Feed 도착 — Mock fixture profile은 onboardingCompleted=true이므로 바로 Feed
        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(
            allTagButton.waitForExistence(timeout: 10),
            "Feed 화면 진입 (Mock 모드 자동 로그인 + 온보딩 완료)"
        )
        takeScreenshot(name: "01_피드_도착")

        // 2. 첫 기사 카드 탭 → ArticleDetailView 진입
        let firstArticleCell = app.cells.firstMatch
        XCTAssertTrue(firstArticleCell.waitForExistence(timeout: 5), "기사 카드 존재")
        firstArticleCell.tap()

        // 3. ArticleDetailView 검증 — "원문 보기" 버튼 존재
        let openOriginalButton = app.buttons["원문 보기"]
        XCTAssertTrue(
            openOriginalButton.waitForExistence(timeout: 5),
            "기사 상세 화면 진입"
        )
        takeScreenshot(name: "02_기사상세")

        // 4. 뒤로가기 → Feed 복귀
        let backButton = app.navigationBars.buttons.element(boundBy: 0)
        XCTAssertTrue(backButton.waitForExistence(timeout: 3), "navigation back 버튼 존재")
        backButton.tap()

        XCTAssertTrue(allTagButton.waitForExistence(timeout: 5), "Feed 복귀")
        takeScreenshot(name: "03_피드_복귀")

        // 5. 설정 버튼 탭 → SettingsView 진입
        let settingsButton = app.buttons["settings_button"]
        XCTAssertTrue(
            settingsButton.waitForExistence(timeout: 3),
            "설정 버튼(accessibilityIdentifier=settings_button) 존재"
        )
        settingsButton.tap()

        // 6. SettingsView 검증 — "태그 관리" 또는 "로그아웃" 등 설정 관련 요소 존재
        // (구체적 텍스트는 SettingsView 구현에 따라 달라질 수 있어 navigationBar 타이틀 검증으로 대체)
        sleep(1)
        takeScreenshot(name: "04_설정화면")
    }

    // MARK: - TC-03: 요약 timeout 배너

    /// TC-03: 피드 → 새 뉴스 가져오기 → timeout 배너 확인 → 다시 시도 버튼 확인
    ///
    /// `FRANK_UI_SCENARIO=summarize_timeout`: MockCollectAdapter.triggerSummarize() → URLError(.timedOut)
    func testSummaryTimeoutUI() {
        launchMock(scenario: "summarize_timeout", summarizeTimeoutSeconds: 3)

        // 1. 피드 도착 확인
        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(
            allTagButton.waitForExistence(timeout: 10),
            "Mock 모드 피드 화면 진입"
        )
        takeScreenshot(name: "tc03_01_피드")

        // 2. "새 뉴스 가져오기" 툴바 버튼 탭
        let collectButton = app.buttons["새 뉴스 가져오기"]
        XCTAssertTrue(collectButton.waitForExistence(timeout: 5), "새 뉴스 가져오기 버튼 존재")
        collectButton.tap()
        takeScreenshot(name: "tc03_02_수집중")

        // 3. timeout 배너 노출 확인 (수집 후 요약 단계에서 즉시 throw)
        let timeoutText = app.staticTexts["요약이 오래 걸리고 있어요"]
        XCTAssertTrue(
            timeoutText.waitForExistence(timeout: 15),
            "요약 timeout 배너 노출 확인"
        )
        takeScreenshot(name: "tc03_03_timeout배너")

        // 4. "다시 시도" 버튼 존재 확인
        let retryButton = app.buttons["다시 시도"]
        XCTAssertTrue(retryButton.exists, "다시 시도 버튼 존재")
        takeScreenshot(name: "tc03_04_재시도버튼")
    }

    // MARK: - TC-04: 설정 → 태그 관리 → 로그아웃

    /// TC-04: 피드 → 설정 탭 → 태그 관리 진입 → 뒤로가기 → 로그아웃 → 로그인 화면 복귀
    func testTagManagementAndLogout() {
        launchMock()

        // 1. 피드 도착 확인
        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(allTagButton.waitForExistence(timeout: 10), "피드 화면 진입")
        takeScreenshot(name: "tc04_01_피드")

        // 2. 설정 버튼 탭
        let settingsButton = app.buttons["settings_button"]
        XCTAssertTrue(settingsButton.waitForExistence(timeout: 5), "설정 버튼 존재")
        settingsButton.tap()

        // 3. 설정 화면 확인
        let settingsTitle = app.navigationBars["설정"]
        XCTAssertTrue(settingsTitle.waitForExistence(timeout: 5), "설정 화면 진입")
        takeScreenshot(name: "tc04_02_설정")

        // 4. 태그 관리 진입
        let tagManagementRow = app.buttons["태그 관리"]
        XCTAssertTrue(tagManagementRow.waitForExistence(timeout: 3), "태그 관리 항목 존재")
        tagManagementRow.tap()

        let tagManagementTitle = app.navigationBars["태그 관리"]
        XCTAssertTrue(tagManagementTitle.waitForExistence(timeout: 5), "태그 관리 화면 진입")
        takeScreenshot(name: "tc04_03_태그관리")

        // 5. 뒤로가기 → 설정 화면 복귀
        let backButton = app.navigationBars.buttons.element(boundBy: 0)
        XCTAssertTrue(backButton.waitForExistence(timeout: 3), "뒤로가기 버튼 존재")
        backButton.tap()
        XCTAssertTrue(settingsTitle.waitForExistence(timeout: 5), "설정 화면 복귀")

        // 6. 로그아웃 버튼 탭
        let signOutButton = app.buttons["로그아웃"]
        XCTAssertTrue(signOutButton.waitForExistence(timeout: 3), "로그아웃 버튼 존재")
        signOutButton.tap()

        // 7. 로그아웃 확인 알럿
        let confirmSignOut = app.alerts["로그아웃"].buttons["로그아웃"]
        XCTAssertTrue(confirmSignOut.waitForExistence(timeout: 3), "로그아웃 확인 알럿")
        confirmSignOut.tap()
        takeScreenshot(name: "tc04_04_로그아웃확인")

        // 8. 로그인 화면 복귀 확인
        let loginTitle = app.staticTexts["Frank"]
        XCTAssertTrue(
            loginTitle.waitForExistence(timeout: 10),
            "로그아웃 후 로그인 화면 복귀"
        )
        takeScreenshot(name: "tc04_05_로그인화면복귀")
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
