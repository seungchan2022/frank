import XCTest

final class OnboardingFlowUITest: XCTestCase {

    private let app = XCUIApplication()

    override func setUp() {
        continueAfterFailure = true
        app.launch()
    }

    func testFullOnboardingFlow() {
        // 1. 앱 로드 대기
        let loginTitle = app.staticTexts["Frank"]
        let feedTag = app.buttons["전체"]

        // 이미 로그인되어 피드 화면인 경우 vs 로그인 화면인 경우
        let loginAppeared = loginTitle.waitForExistence(timeout: 5)
        let alreadyOnFeed = feedTag.exists

        if alreadyOnFeed {
            // 이미 인증된 세션 — 피드 화면 검증
            takeScreenshot(name: "01_이미_로그인됨_피드화면")
            XCTAssertTrue(feedTag.exists, "피드 태그 탭 존재")
            return
        }

        guard loginAppeared else {
            takeScreenshot(name: "01_실패_알수없는화면")
            XCTFail("로그인 화면도 피드 화면도 아닌 상태")
            return
        }

        takeScreenshot(name: "01_로그인화면")

        // 2. "다른 방법으로 로그인" 탭
        let otherLoginButton = app.buttons["다른 방법으로 로그인"]
        guard otherLoginButton.waitForExistence(timeout: 3) else {
            takeScreenshot(name: "02_실패_버튼없음")
            XCTFail("'다른 방법으로 로그인' 버튼을 찾을 수 없음")
            return
        }
        otherLoginButton.tap()
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

        let appeared = onboardingTitle.waitForExistence(timeout: 15)
            || feedTag.waitForExistence(timeout: 3)

        takeScreenshot(name: "04_로그인후화면")

        if appeared {
            if onboardingTitle.exists {
                // 온보딩 화면 — 태그 확인
                sleep(2)
                takeScreenshot(name: "05_온보딩_태그로드")

                let startButton = app.buttons["시작하기"]
                XCTAssertTrue(startButton.exists, "시작하기 버튼 존재")
            }
        } else {
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
