# M4: iOS 테스트 인프라

> 프로젝트: Frank MVP4
> 상태: ✅ 완료 (260408)
> 예상 기간: 2~3일
> 의존성: M2 완료 후 (LoginView 변경 안정화 후 TC-01 작성 가능)
> 실행: `/workflow "MVP4 M4: iOS 테스트 인프라"`

---

## 목표

iOS 테스트 인프라를 구축한다.
커버리지 측정을 자동화하고, 핵심 E2E 시나리오 4개를 XCUITest로 자동화한다.

---

## 서브태스크

| # | 서브태스크 | 유형 | 비고 |
|---|-----------|------|------|
| 1 | iOS 커버리지 측정 파이프라인 | chore | scripts/coverage.sh |
| 2 | UITest TC-01: 이메일 로그인 → 피드 진입 | feature | M2 완료 후 안정적 작성 가능 |
| 3 | UITest TC-02: 온보딩 플로우 (신규 사용자) | feature | |
| 4 | UITest TC-03: 피드 → 새 뉴스 가져오기 → timeout 배너 확인 | feature | timeout mock scenario 필요 |
| 5 | UITest TC-04: 설정 → 태그 관리 → 로그아웃 | feature | |

---

## 성공 기준 (DoD)

- [x] `scripts/coverage.sh` 실행으로 iOS 커버리지 수치 출력 (90% 미만 exit 1)
- [x] TC-01~04 XCUITest 시나리오 전체 통과 (시뮬레이터 기준)
- [x] 기존 iOS 테스트 155개+ 전체 통과 (181개 통과)

---

## 서브태스크 상세

### ST-1: 커버리지 파이프라인

```bash
# scripts/coverage.sh
# xcodebuild test -enableCodeCoverage YES
# xcrun xccov view --report --json → 수치 파싱 + 90% 미만 시 경고
```

---

### ST-0 (선행): Mock Scenario 주입 설계

UITest TC-01/02/03 실행 전 반드시 선행해야 한다.

```swift
// AppDependencies.swift — launchEnvironment 기반 분기
// FRANK_UI_SCENARIO=logged_out   → MockAuthAdapter.currentSession() = nil
// FRANK_UI_SCENARIO=new_user     → MockFixtures.profile.onboardingCompleted = false
// FRANK_UI_SCENARIO=timeout      → MockCollectAdapter 지연 + timeout 응답
```

```swift
// 각 UITest setUp
app.launchEnvironment["FRANK_USE_MOCK"] = "1"
app.launchEnvironment["FRANK_UI_SCENARIO"] = "logged_out"  // TC-01용
```

---

### ST-2~5: UITest 시나리오

```
TC-01: FRANK_UI_SCENARIO=logged_out → 로그인 화면 → 이메일/비밀번호 입력 → 피드 표시 확인
TC-02: FRANK_UI_SCENARIO=new_user → 태그 선택 화면 → 최소 1개 선택 → 피드 진입
TC-03: FRANK_UI_SCENARIO=timeout → 피드 → 새 뉴스 가져오기 버튼 탭 → timeout 배너 확인 → 재시도 버튼 확인
TC-04: FRANK_USE_MOCK=1 (기본) → 설정 탭 → 태그 추가/삭제 → 로그아웃 → 로그인 화면 복귀
```

> Apple 로그인은 XCUITest Mock 어려움 → 전 시나리오 이메일 로그인 기준.
> TC-03: 기존 스펙의 "상세 화면 요약 버튼"은 앱에 없음 — 피드 timeout 배너로 수정 (리뷰 결과 반영).

---

## 리뷰 결과 (Step-5)

> 리뷰 일자: 260408

### Claude 리뷰 (문서 일치성)

**전체 평가**: 조건부 승인 — TC-03 스펙 수정 필수, mock scenario 설계 선행 필요

**문서 일치성 이슈**:

1. **TC-03 스펙 vs 실제 앱 불일치** (심각)
   - 문서: "기사 상세 → 요약 버튼 → timeout UI"
   - 실제: 요약 트리거는 피드 툴바의 "새 뉴스 가져오기", timeout UI는 피드 배너 (`FeedView.swift`)
   - `ArticleDetailView.swift`에는 요약 버튼이 없음
   - **→ TC-03 시나리오를 "피드 → 새 뉴스 가져오기 → timeout 배너 확인"으로 수정 필요**

2. **TC-01/TC-02 mock 기본 상태와 충돌** (중요)
   - `MockAuthAdapter.currentSession()`: 항상 profile 반환 → 로그인 화면 노출 불가
   - `MockFixtures.profile`: `onboardingCompleted=true` → 신규 사용자 온보딩 흐름 진입 불가
   - **→ `FRANK_UI_SCENARIO` 환경변수 기반 mock scenario 주입 설계 선행 필요**

3. **기존 UITest 품질 낮음** (보완)
   - `OnboardingFlowUITest.swift`에 `continueAfterFailure=true`, `sleep(1)`, 분기 허용 구조
   - 회귀 검출력 약함 → M4에서 같이 정비 권장

4. **접근성 식별자 부족**
   - `settings_button` 외 대부분 텍스트 기반 탐색
   - 로그인 화면, 피드 툴바 버튼 등에 `accessibilityIdentifier` 보강 필요

**DoD 항목 검토**:
- `scripts/coverage.sh`: `xcresult` 기반 집계, app-target 필터링 명시 필요 (외부 프레임워크 분모 혼입 위험)
- "90% 이상" 목표: 측정 파이프라인 확보가 우선 — 1차 DoD는 "수치 출력"으로 설정이 현실적

---

### Codex 리뷰 (기술적 타당성)

**전체 평가**: 조건부 승인 — 인프라 자체는 기존 코드 재활용 가능, mock scenario 확장이 핵심 선행작업

**ST-1 (커버리지 파이프라인)**: 실현 가능
- `FrankUITests` 타겟은 이미 `Project.swift`에 선언됨 (신규 추가 불필요)
- 필요 플래그: `-workspace Frank.xcworkspace -scheme Frank -destination ... -resultBundlePath ...`
- `xcrun xccov view --report --json <xcresult>` 후 앱 타겟만 필터링 파싱

**ST-2 (TC-01 이메일 로그인)**: mock scenario 추가 선행 필요
- `MockAuthAdapter.currentSession()` 항상 로그인 반환 → 로그인 화면 진입 불가
- `FRANK_UI_SCENARIO=logged_out` 또는 `FRANK_AUTH_STATE=signed_out` 주입 필요

**ST-3 (TC-02 온보딩)**: mock scenario 추가 선행 필요
- `MockFixtures.profile.onboardingCompleted=true` → 온보딩 진입 불가
- `FRANK_ONBOARDING_COMPLETED=false` 환경변수 주입 또는 별도 fixture 필요

**ST-4 (TC-03 요약 timeout)**: 스펙 수정 먼저
- 앱에 "상세 화면 요약 버튼" 없음
- `MockCollectAdapter` 항상 즉시 성공 → timeout 상태 재현 불가
- timeout mock scenario (`FRANK_SUMMARIZE_MODE=timeout`) 필요

**ST-5 (TC-04 설정/태그/로그아웃)**: 현재 구조로 진행 가능
- 설정/태그/로그아웃 UI 존재, `settings_button` accessibilityIdentifier 있음
- "앱 재실행 후 로그아웃 유지" 검증까지 하려면 mock auth 개선 필요

---

### 최종 결정: 조건부 승인

**구현 전 필수 선행 작업**:

1. **TC-03 시나리오 재정의** — "피드 → 새 뉴스 가져오기 → timeout 배너 확인"으로 변경
2. **Mock Scenario 주입 설계** — `FRANK_UI_SCENARIO` 환경변수 기반:
   - `logged_out`: `MockAuthAdapter.currentSession()` → nil 반환
   - `new_user`: `MockFixtures.profile.onboardingCompleted=false`
   - `summarize_timeout`: `MockCollectAdapter` 지연 후 timeout 응답
3. **접근성 식별자 보강** — 로그인 버튼, 피드 툴바 버튼, 이메일 필드 등

**수정 후 각 ST 실행 순서**:
- ST-1 → Mock Scenario 설계 (선행) → ST-2 → ST-3 → ST-4 → ST-5
