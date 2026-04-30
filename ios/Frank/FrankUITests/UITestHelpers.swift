import XCTest

/// UITest 공통 헬퍼 — FrankUITests 전용
extension XCTestCase {
    /// 스크린샷을 XCTAttachment로 첨부 (항상 유지)
    func takeScreenshot(app: XCUIApplication, name: String) {
        let screenshot = app.windows.firstMatch.screenshot()
        let attachment = XCTAttachment(screenshot: screenshot)
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}

extension XCUIApplication {
    /// Mock 모드로 앱 실행 (`FRANK_USE_MOCK=1`)
    ///
    /// - Parameter scenario: `FRANK_UI_SCENARIO` 값. nil 이면 키를 **제거**하여 이전 값 누적 방지.
    func launchMock(scenario: String? = nil) {
        launchEnvironment["FRANK_USE_MOCK"] = "1"
        if let scenario {
            launchEnvironment["FRANK_UI_SCENARIO"] = scenario
        } else {
            launchEnvironment.removeValue(forKey: "FRANK_UI_SCENARIO")
        }
        launch()
    }
}
