# E2E 리포트 예시 (포맷 기준)

> 이 파일은 E2E 리포트 포맷을 보여주는 빈 예시입니다.
> 실제 시나리오 실행 리포트는 M4 이후 생성됩니다.

---

# E2E 실행 리포트

> 날짜: YYYY-MM-DD
> 실행자: /e2e 스킬
> 브랜치: feature/xxx
> 환경: local | staging

## 요약

| 플랫폼 | 전체 | 통과 | 실패 | 스킵 |
|--------|------|------|------|------|
| 웹 (Playwright) | 0 | 0 | 0 | 0 |
| iOS (XCUITest) | 4 | 4 | 0 | 0 |
| **합계** | **4** | **4** | **0** | **0** |

## 웹 E2E (Playwright)

| # | 시나리오 | 파일 | 결과 | 시간 |
|---|---------|------|------|------|
| 1 | smoke: Playwright 실행 환경 동작 확인 | e2e/smoke.spec.ts | ✅ PASS | -ms |

_실제 시나리오는 M4에서 추가_

## iOS E2E (XCUITest)

| # | 클래스 | 메서드 | 결과 | 시간 |
|---|--------|--------|------|------|
| 1 | CrossFeatureFlowUITest | testFeedToDetailToSettingsFlow | ✅ PASS | -s |
| 2 | CrossFeatureFlowUITest | testTagManagementAndLogout | ✅ PASS | -s |
| 3 | LoginFlowUITest | testEmailLoginToFeed | ✅ PASS | -s |
| 4 | OnboardingFlowUITest | testNewUserOnboardingFlow | ✅ PASS | -s |

## 실패 항목 상세

_없음_

## 환경 정보

- BASE_URL: http://localhost:5173
- iOS 시뮬레이터: iPhone 17 Pro
- Playwright 버전: `npx playwright --version`으로 확인
- Xcode 버전: `xcodebuild -version`으로 확인

## 비고

_없음_
