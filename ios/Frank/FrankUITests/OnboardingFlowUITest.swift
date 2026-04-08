import XCTest

/// TC-02: 신규 사용자 온보딩 플로우
///
/// 시나리오: FRANK_USE_MOCK=1 + FRANK_UI_SCENARIO=new_user
/// → onboardingCompleted=false 프로필 → 온보딩 화면 → 태그 선택 → 시작하기 → 피드 진입
final class OnboardingFlowUITest: XCTestCase {

    private let app = XCUIApplication()

    override func setUp() {
        continueAfterFailure = false
        app.launchEnvironment["FRANK_USE_MOCK"] = "1"
        app.launchEnvironment["FRANK_UI_SCENARIO"] = "new_user"
        app.launch()
    }

    func testNewUserOnboardingFlow() {
        // 1. 온보딩 화면 도착 확인
        let onboardingTitle = app.staticTexts["관심 키워드를 선택하세요"]
        XCTAssertTrue(
            onboardingTitle.waitForExistence(timeout: 10),
            "신규 사용자이므로 온보딩 화면이 노출되어야 함"
        )
        takeScreenshot(name: "01_온보딩화면")

        // 2. 태그 로드 대기 후 첫 번째 태그 선택
        let firstTag = app.buttons.matching(NSPredicate(format: "label != '시작하기'")).firstMatch
        XCTAssertTrue(firstTag.waitForExistence(timeout: 5), "태그 목록 로드 확인")
        firstTag.tap()
        takeScreenshot(name: "02_태그선택")

        // 3. 시작하기 버튼 활성화 확인 후 탭
        let startButton = app.buttons["시작하기"]
        XCTAssertTrue(startButton.waitForExistence(timeout: 3), "시작하기 버튼 존재")
        startButton.tap()

        // 4. 피드 화면 진입 확인
        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(
            allTagButton.waitForExistence(timeout: 10),
            "온보딩 완료 후 피드 화면 진입 확인"
        )
        takeScreenshot(name: "03_피드화면_진입")
    }

    private func takeScreenshot(name: String) {
        let screenshot = app.windows.firstMatch.screenshot()
        let attachment = XCTAttachment(screenshot: screenshot)
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
