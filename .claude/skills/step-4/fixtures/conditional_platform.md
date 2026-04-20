# Fixture: 조건부 카테고리 포함 (iOS UI·UX + 플랫폼)

> 용도: 조건부 카테고리(U, P) 생성 검증 + 4상태 모델 샘플
> 가정 서브태스크: iOS 피드 화면 SwiftUI 신규 구현 (파일 5개)

## Feature List
<!-- size: 중형 | count: 22 | skip: false -->

### 기능
- [ ] F-01 FeedView가 루트 탭에서 정상 표시된다
- [ ] F-02 SwiftUI List로 뉴스 목록 렌더링
- [ ] F-03 스크롤 페이지네이션 동작 (20개씩 로드)
- [ ] F-04 Pull-to-refresh 동작
- [ ] F-05 카드 탭 시 상세 화면 push navigation
- [x] F-06 로딩 중 ProgressView 표시
- [~] deferred (위젯 추가는 다음 마일스톤) F-07 홈 화면 위젯 지원

### 엣지
- [ ] E-01 빈 상태 Placeholder 뷰
- [ ] E-02 오프라인 상태 감지 + 배너 표시
- [-] N/A (이번 서브태스크는 신규 구현이라 회전 대응 불필요) E-03 가로 모드 대응

### 에러
- [ ] R-01 API 실패 시 에러 바 표시
- [ ] R-02 Keychain 접근 실패 시 재인증 유도

### 테스트
- [ ] T-01 XCTest 단위 테스트: ViewModel 상태 전이
- [ ] T-02 XCTest 단위 테스트: Pagination 로직
- [ ] T-03 XCUITest E2E: 앱 시작 → 피드 → 상세
- [ ] T-04 커버리지 90% 이상

### UI·UX
- [ ] U-01 다크모드 대응 (SwiftUI `@Environment(\.colorScheme)`)
- [ ] U-02 Dynamic Type 텍스트 크기 대응
- [ ] U-03 VoiceOver 라벨 설정

### 플랫폼
- [ ] P-01 Tuist generate 후 project.pbxproj 커밋 포함
- [ ] P-02 iOS 17.0 최소 타겟 확인
- [ ] P-03 Release 빌드 아카이브 성공
- [ ] P-04 TestFlight 업로드 리허설
