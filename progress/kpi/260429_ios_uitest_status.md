# iOS UITest 현황 기록 (ST-1)

> 날짜: 2026-04-29
> 목적: M1 기준선 확보 — E2E 인프라 세팅 전 현재 상태 기록
> 실행 환경: iPhone 17 Pro Simulator, Xcode

## 실행 명령

```bash
cd ios/Frank
~/.tuist/Versions/4.31.0/tuist generate --no-open
xcodebuild test -workspace Frank.xcworkspace -scheme Frank \
  -destination 'platform=iOS Simulator,name=iPhone 17 Pro' \
  -only-testing:FrankUITests
```

## 결과 요약

**전체: 4/4 PASS (0 failures)**

| # | 테스트 클래스 | 테스트 메서드 | 결과 | 소요 시간 |
|---|-------------|------------|------|----------|
| 1 | CrossFeatureFlowUITest | testFeedToDetailToSettingsFlow | ✅ PASS | 15.3s |
| 2 | CrossFeatureFlowUITest | testTagManagementAndLogout | ✅ PASS | 20.6s |
| 3 | LoginFlowUITest | testEmailLoginToFeed | ✅ PASS | 16.7s |
| 4 | OnboardingFlowUITest | testNewUserOnboardingFlow | ✅ PASS | 8.8s |

**전체 실행 시간: 약 61.4초**

## 파일 위치

```
ios/Frank/FrankUITests/
├── CrossFeatureFlowUITest.swift   — 2개 테스트 (크로스 피처 플로우)
├── LoginFlowUITest.swift          — 1개 테스트 (이메일 로그인)
└── OnboardingFlowUITest.swift     — 1개 테스트 (신규 유저 온보딩)
```

### AuthFlowUITest.swift 위치 이슈 (F-07 / P-03)

- **현재 위치**: `ios/Frank/FrankTests/Features/AuthFlowUITest.swift`
- **타겟**: FrankTests (유닛/통합 테스트 타겟)
- **내용**: XCUITest API 미사용 — `@testable import Frank` + MockAuthPort 기반 통합 테스트
- **내부 Suite 이름**: `@Suite("Auth Flow Integration Tests")` → 파일명과 불일치
- **결정**: FrankTests 타겟 유지가 올바름. XCUITest가 아닌 단위/통합 테스트.
- **파일명 불일치 트레이드오프**: 파일명 `AuthFlowUITest.swift`는 UITest처럼 보여 혼란 유발 가능.
  - 옵션 A: `AuthFlowIntegrationTests.swift`으로 rename → Suite 이름과 일치
  - 옵션 B: 파일 상단 주석으로 의도 명시 → 파일 이동 없이 해결
  - **M1 결정**: 문서 기록만. 실제 rename은 step-7(리팩토링) 단계에서 진행 예정.
- UITest처럼 보이는 파일명이지만 실제로는 Mock 기반 통합 테스트 → FrankUITests 이동 불필요

## M4 신규 UITest 추가 시 위치 규칙

- **XCUITest (실제 UI 조작)**: `ios/Frank/FrankUITests/` — Project.swift glob `FrankUITests/**` 자동 커버
- **Mock 기반 통합 테스트**: `ios/Frank/FrankTests/Features/` — Project.swift glob `FrankTests/**` 자동 커버
- Tuist Project.swift 수정 불필요 (Sources glob이 해당 디렉토리 전체 커버)

## 비고

- `** TEST SUCCEEDED **` 확인
- 시뮬레이터 기동 포함 전체 94초 소요
