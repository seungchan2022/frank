# ST-9: C-2 XCUITest rewrite 버튼→결과 표시

> 브랜치: feature/260504_feed-quality-debt-cleanup  
> 파일: ios/Frank/FrankUITests/RewriteFlowUITest.swift (신규)

## 요약
iOS에서 재작성(rewrite) 버튼 탭 후 결과 화면 표시 XCUITest 검증.

## Feature List
<!-- size: 중형 | count: 14 | skip: false -->

### 기능
- [x] F-01 UITestHelpers 패턴으로 시뮬레이터 로그인
- [x] F-02 피드 첫 기사 탭 → 상세 화면 진입
- [x] F-03 요약 버튼 탭 → AI 요약 완료 대기 (waitForExistence)
- [x] F-04 재작성 버튼 탭 → 재작성 결과 텍스트 뷰 표시 확인 (accessibilityIdentifier 기반)

### 엣지
- [x] E-01 요약 완료 전 재작성 버튼 비활성화 확인
- [x] E-02 재작성 결과 텍스트가 비어있지 않음 확인

### 에러
- [ ] R-01 네트워크 오류 시 에러 알럿 또는 에러 메시지 표시 확인

### 플랫폼
- [x] P-01 accessibilityIdentifier "rewriteResultText" 뷰가 ArticleDetailView에 설정됨
- [x] P-02 iPhone 17 Pro 시뮬레이터에서 정상 실행
- [x] P-03 xcodebuild test -workspace Frank.xcworkspace -scheme Frank 통과

### 테스트
- [x] T-01 전체 RewriteFlowUITest 시나리오 pass
- [x] T-02 기존 UITest(LoginFlow, FeedRefresh 등) 회귀 없음
- [x] T-03 시뮬레이터 상태 독립 (이전 테스트 상태 미의존)
- [x] T-04 테스트 실행 시간 60초 이내
