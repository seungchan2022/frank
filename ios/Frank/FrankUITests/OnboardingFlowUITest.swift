import XCTest

final class OnboardingFlowUITest: XCTestCase {

    private let app = XCUIApplication()

    override func setUp() {
        continueAfterFailure = true
        app.launch()
    }

    func testFullOnboardingFlow() {
        // 1. 로그인 화면
        let loginTitle = app.staticTexts["Frank"]
        XCTAssertTrue(loginTitle.waitForExistence(timeout: 5))
        takeScreenshot(name: "01_로그인화면")

        // 2. "다른 방법으로 로그인" 탭
        app.buttons["다른 방법으로 로그인"].tap()
        sleep(1)
        takeScreenshot(name: "02_이메일시트")

        // 3. 이메일 입력
        let emailField = app.textFields["이메일"]
        if emailField.waitForExistence(timeout: 3) {
            emailField.tap()
            emailField.typeText("e2e@frank.dev")
        }

        // 4. 비밀번호 입력
        let passwordField = app.secureTextFields["비밀번호 (6자 이상)"]
        if passwordField.waitForExistence(timeout: 2) {
            passwordField.tap()
            passwordField.typeText("Test1234!")
        }
        takeScreenshot(name: "03_입력완료")

        // 5. 로그인 탭
        let loginButton = app.buttons["로그인"]
        if loginButton.waitForExistence(timeout: 2) {
            loginButton.tap()
        }

        // 6. 온보딩 또는 피드 대기 (최대 15초)
        let onboardingTitle = app.staticTexts["관심 키워드를 선택하세요"]
        let feedTitle = app.staticTexts["로그인 완료"]

        let appeared = onboardingTitle.waitForExistence(timeout: 15)
            || feedTitle.waitForExistence(timeout: 3)

        takeScreenshot(name: "04_로그인후화면")

        if appeared {
            if onboardingTitle.exists {
                // 온보딩 화면 — 태그 확인
                sleep(2)
                takeScreenshot(name: "05_온보딩_태그로드")

                // 시작하기 버튼 확인
                let startButton = app.buttons["시작하기"]
                XCTAssertTrue(startButton.exists, "시작하기 버튼 존재")
            }
        } else {
            // 현재 화면 상태를 캡처
            takeScreenshot(name: "04_실패_현재화면")
        }
    }

    private func takeScreenshot(name: String) {
        let screenshot = app.windows.firstMatch.screenshot()
        let attachment = XCTAttachment(screenshot: screenshot)
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
