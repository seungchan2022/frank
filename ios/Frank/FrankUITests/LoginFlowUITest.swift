import XCTest

/// TC-01: 이메일 로그인 → 피드 진입
///
/// 시나리오: FRANK_USE_MOCK=1 + FRANK_UI_SCENARIO=logged_out
/// → 로그아웃 상태 → 이메일 로그인 시트 → Mock 로그인 → 피드 화면 확인
final class LoginFlowUITest: XCTestCase {

    private let app = XCUIApplication()

    override func setUp() {
        continueAfterFailure = false
        app.launchEnvironment["FRANK_USE_MOCK"] = "1"
        app.launchEnvironment["FRANK_UI_SCENARIO"] = "logged_out"
        app.launch()
    }

    func testEmailLoginToFeed() {
        // 1. 로그인 화면 도착 확인
        let loginTitle = app.staticTexts["Frank"]
        XCTAssertTrue(
            loginTitle.waitForExistence(timeout: 10),
            "로그아웃 상태이므로 로그인 화면이 노출되어야 함"
        )
        takeScreenshot(name: "01_로그인화면")

        // 2. "다른 방법으로 로그인" 탭
        let otherLoginButton = app.buttons["다른 방법으로 로그인"]
        XCTAssertTrue(otherLoginButton.waitForExistence(timeout: 5), "'다른 방법으로 로그인' 버튼 존재")
        otherLoginButton.tap()
        takeScreenshot(name: "02_이메일시트_열림")

        // 3. 이메일 입력
        let emailField = app.textFields["이메일"]
        XCTAssertTrue(emailField.waitForExistence(timeout: 5), "이메일 입력 필드 존재")
        emailField.tap()
        emailField.typeText("mock@frank.dev")

        // 4. 비밀번호 입력
        let passwordField = app.secureTextFields["비밀번호 (6자 이상)"]
        XCTAssertTrue(passwordField.waitForExistence(timeout: 3), "비밀번호 입력 필드 존재")
        passwordField.tap()
        passwordField.typeText("mock1234")
        takeScreenshot(name: "03_입력완료")

        // 5. 로그인 버튼 탭
        let loginButton = app.buttons["로그인"]
        XCTAssertTrue(loginButton.waitForExistence(timeout: 3), "로그인 버튼 존재")
        loginButton.tap()

        // 6. 피드 화면 진입 확인 (Mock 로그인은 onboardingCompleted=true)
        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(
            allTagButton.waitForExistence(timeout: 10),
            "로그인 후 피드 화면 진입 확인"
        )
        takeScreenshot(name: "04_피드화면")
    }

    private func takeScreenshot(name: String) {
        let screenshot = app.windows.firstMatch.screenshot()
        let attachment = XCTAttachment(screenshot: screenshot)
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
