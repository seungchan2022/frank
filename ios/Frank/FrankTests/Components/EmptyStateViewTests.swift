import Testing
import Foundation
@testable import Frank

@Suite("EmptyStateView 데이터 바인딩")
struct EmptyStateViewTests {

    @Test("기본 메시지 확인")
    func defaultMessage() {
        let view = EmptyStateView()

        #expect(view.message == "이 키워드의 뉴스가 아직 없습니다")
    }

    @Test("기본 아이콘 확인")
    func defaultIcon() {
        let view = EmptyStateView()

        #expect(view.icon == "newspaper")
    }

    @Test("커스텀 메시지 전달")
    func customMessage() {
        let view = EmptyStateView(message: "검색 결과가 없습니다")

        #expect(view.message == "검색 결과가 없습니다")
    }

    @Test("커스텀 아이콘 전달")
    func customIcon() {
        let view = EmptyStateView(icon: "magnifyingglass")

        #expect(view.icon == "magnifyingglass")
    }

    @Test("아이콘과 메시지 모두 커스텀")
    func customIconAndMessage() {
        let view = EmptyStateView(
            icon: "exclamationmark.triangle",
            message: "오류가 발생했습니다"
        )

        #expect(view.icon == "exclamationmark.triangle")
        #expect(view.message == "오류가 발생했습니다")
    }
}
