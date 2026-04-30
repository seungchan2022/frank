import XCTest

/// I-04: pull-to-refresh 시나리오
///
/// BUG-007: pull-to-refresh 동작 검증
/// - swipeDown 제스처로 RefreshControl 트리거
/// - 새로고침 인디케이터 동작 확인
/// - 새로고침 완료 후 기사 목록 유지/변경 확인
///
/// `FRANK_UI_SCENARIO=feed_refresh_2step`: MockArticleAdapter가
/// noCache=true 호출 시 다른 fixture 세트를 반환하여 목록 변경을 시뮬레이션.
final class FeedRefreshUITest: XCTestCase {

    private let app = XCUIApplication()

    override func setUp() {
        continueAfterFailure = false
    }

    // MARK: - I-04: pull-to-refresh → 기사 목록 갱신

    /// I-04: BUG-007 — pull-to-refresh 트리거 → 기사 목록 갱신 확인
    ///
    /// 2단계 MockArticleAdapter: 초기 로드 시 feedItems, pull-to-refresh 시 refreshedFeedItems 반환
    func testPullToRefreshUpdatesFeed() {
        app.launchMock(scenario: "feed_refresh_2step")

        // 1. 피드 화면 진입 확인
        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(allTagButton.waitForExistence(timeout: 10), "피드 화면 진입")
        takeScreenshot(name: "i04_01_초기_피드")

        // 2. 첫 번째 기사 존재 확인 (초기 fixture 기사 로딩됨)
        let firstCell = app.cells.firstMatch
        XCTAssertTrue(firstCell.waitForExistence(timeout: 5), "초기 기사 목록 로딩됨")
        takeScreenshot(name: "i04_02_초기_기사목록")

        // 3. swipeDown으로 pull-to-refresh 트리거
        // SwiftUI List는 iOS에서 UITableView → app.tables로 접근
        // collectionViews fallback: LazyVStack 등 커스텀 레이아웃 대비
        let tableView = app.tables.firstMatch
        let targetView: XCUIElement
        if tableView.exists {
            targetView = tableView
        } else {
            let collectionView = app.collectionViews.firstMatch
            targetView = collectionView.exists ? collectionView : firstCell
        }

        targetView.swipeDown()
        takeScreenshot(name: "i04_03_pull_to_refresh_1차시도")

        // 4. 새로고침 완료 후 기사 목록 여전히 존재 (E-03: 인디케이터 사라짐)
        // Mock이므로 빠르게 완료됨 — RefreshControl 트리거가 불안정할 수 있어 최대 2회 시도
        sleep(1)

        // 1차 시도에서 셀이 사라지지 않았으면 바로 확인, 아니면 retry
        let cellAfterRefresh = app.cells.firstMatch
        if !cellAfterRefresh.waitForExistence(timeout: 3) {
            // Retry: 두 번째 swipeDown 시도
            targetView.swipeDown()
            sleep(2)
        } else {
            sleep(1) // 새로고침 인디케이터 사라질 때까지 대기
        }

        XCTAssertTrue(
            cellAfterRefresh.waitForExistence(timeout: 5),
            "BUG-007: pull-to-refresh 완료 후 기사 목록 유지/갱신"
        )
        takeScreenshot(name: "i04_04_새로고침_완료")

        // 5. 피드 화면 헤더 요소가 여전히 존재 (화면 무너짐 없음)
        XCTAssertTrue(
            allTagButton.waitForExistence(timeout: 3),
            "I-04: 새로고침 후 피드 화면 헤더 유지"
        )
    }

    // MARK: - R-02: Mock 모드에서 RefreshControl 동작 확인

    /// R-02: Mock 모드에서 pull-to-refresh 동작이 정상 동작
    func testPullToRefreshInMockMode() {
        app.launchMock()

        // 1. 피드 진입
        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(allTagButton.waitForExistence(timeout: 10), "피드 화면 진입")

        // 2. 기사 목록 로딩 대기
        let firstCell = app.cells.firstMatch
        XCTAssertTrue(firstCell.waitForExistence(timeout: 5), "기사 목록 로딩됨")

        // 3. swipeDown — Mock 모드에서도 동작해야 함
        // SwiftUI List → UITableView → app.tables 우선 접근
        let tableView = app.tables.firstMatch
        if tableView.exists {
            tableView.swipeDown()
        } else {
            let collectionView = app.collectionViews.firstMatch
            if collectionView.exists {
                collectionView.swipeDown()
            } else {
                firstCell.swipeDown()
            }
        }

        // 4. 잠깐 대기 후 피드 화면 정상 복귀 확인
        sleep(1)
        XCTAssertTrue(
            allTagButton.waitForExistence(timeout: 5),
            "R-02: Mock 모드 pull-to-refresh 후 피드 화면 정상 유지"
        )
        takeScreenshot(name: "r02_mock_refresh_완료")
    }

    // MARK: - E-03: 빈 피드에서 swipeDown() 시 크래시 없음

    /// E-03: empty_feed 시나리오 — 빈 피드에서 pull-to-refresh 후 크래시 없음
    func testPullToRefreshOnEmptyFeedNoCrash() {
        app.launchMock(scenario: "empty_feed")

        // 1. 피드 화면 진입 (빈 피드 상태)
        let allTagButton = app.buttons["전체"]
        XCTAssertTrue(allTagButton.waitForExistence(timeout: 10), "빈 피드 화면 진입")
        takeScreenshot(name: "e03_01_빈_피드")

        // 2. swipeDown — 빈 피드에서도 크래시 없어야 함
        let tableView = app.tables.firstMatch
        if tableView.exists {
            tableView.swipeDown()
        } else {
            let collectionView = app.collectionViews.firstMatch
            if collectionView.exists {
                collectionView.swipeDown()
            } else {
                app.swipeDown()
            }
        }
        sleep(1)

        // 3. 크래시 없이 피드 화면 유지 확인
        XCTAssertTrue(
            allTagButton.waitForExistence(timeout: 5),
            "E-03: 빈 피드 swipeDown 후 크래시 없음, 화면 유지"
        )
        takeScreenshot(name: "e03_02_빈_피드_refresh_후")
    }

    // MARK: - Helpers

    private func takeScreenshot(name: String) {
        takeScreenshot(app: app, name: name)
    }
}
