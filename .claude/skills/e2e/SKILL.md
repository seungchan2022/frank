# /e2e 스킬

> 상태: 뼈대만 (M1). 실제 시나리오는 M4에서 추가.
> 트리거: `/e2e`, `e2e 실행`, `E2E 테스트`

---

## 개요

Frank 프로젝트의 E2E 테스트를 실행하고 결과를 리포트로 기록하는 스킬.

- **웹**: Playwright (Chromium) — `web/e2e/`
- **iOS**: XCUITest — `ios/Frank/FrankUITests/`

---

## 실행 전 필수 조건

```bash
# 1. 서버 + 웹 프론트 선행 기동 (웹 E2E 실행 시 필수)
scripts/deploy.sh --target=api,front --native

# 2. Tuist 프로젝트 생성 (iOS E2E 실행 시 필수)
cd ios/Frank && ~/.tuist/Versions/4.31.0/tuist generate --no-open
```

---

## 실행 명령

### 웹 E2E (Playwright)

```bash
# 전체 실행
cd web && npx playwright test

# 특정 파일만
cd web && npx playwright test e2e/smoke.spec.ts

# BASE_URL 지정 (staging 등)
BASE_URL=http://staging.frank.dev cd web && npx playwright test

# 헤드리스 해제 (디버깅용)
cd web && npx playwright test --headed

# 실패 시 에러 로그 수집
cd web && npx playwright test 2>&1 | tee /tmp/playwright_run.log
```

### iOS E2E (XCUITest)

```bash
# 전체 UITest 실행
cd ios/Frank && xcodebuild test \
  -workspace Frank.xcworkspace \
  -scheme Frank \
  -destination 'platform=iOS Simulator,name=iPhone 17 Pro' \
  -only-testing:FrankUITests

# 특정 클래스만
cd ios/Frank && xcodebuild test \
  -workspace Frank.xcworkspace \
  -scheme Frank \
  -destination 'platform=iOS Simulator,name=iPhone 17 Pro' \
  -only-testing:FrankUITests/LoginFlowUITest

# 결과 필터링
... | grep -E "(Test Case|Test Suite|Executed|SUCCEED|FAILED)"
```

---

## 파일 구조

### 웹 E2E

```
web/
├── playwright.config.ts          — Playwright 설정 (BASE_URL, chromium)
└── e2e/
    ├── smoke.spec.ts             — 더미 smoke 테스트 (인프라 검증용)
    └── (M4 시나리오 추가 예정)
        ├── auth/
        │   └── login.spec.ts
        ├── feed/
        │   └── feed.spec.ts
        └── helpers/
            └── db-reset.ts
```

### iOS E2E

```
ios/Frank/FrankUITests/
├── CrossFeatureFlowUITest.swift  — 크로스 피처 플로우 (2개 테스트)
├── LoginFlowUITest.swift         — 이메일 로그인 플로우 (1개 테스트)
├── OnboardingFlowUITest.swift    — 신규 유저 온보딩 (1개 테스트)
└── (M4 시나리오 추가 위치)
    └── {Feature}FlowUITest.swift
```

**참고**: `FrankTests/Features/AuthFlowUITest.swift`는 Mock 기반 통합 테스트 (XCUITest 아님)

---

## 신규 파일 추가 규칙

### 웹 Playwright 시나리오

1. `web/e2e/{feature}/` 디렉토리 생성
2. `{feature}.spec.ts` 파일 작성
3. Vitest와 경로 충돌 없음 (`src/**` vs `e2e/**` 분리됨)

### iOS XCUITest 시나리오

1. `ios/Frank/FrankUITests/{Feature}FlowUITest.swift` 파일 생성
2. Tuist Project.swift 수정 불필요 (`FrankUITests/**` glob 자동 커버)
3. 클래스명: `{Feature}FlowUITest`, 파일명: `{Feature}FlowUITest.swift`

---

## 리포트 출력

실행 완료 후 `progress/kpi/YYMMDD_e2e_report.md` 파일 생성.

```bash
# 리포트 파일명 규칙
DATE=$(date +%y%m%d)
REPORT_FILE="progress/kpi/${DATE}_e2e_report.md"
```

리포트 포맷: `progress/kpi/260429_e2e_report_example.md` 참조

---

## 에러 처리

| 상황 | 원인 | 조치 |
|------|------|------|
| `playwright test` 실패 — connection refused | 서버 미기동 | `scripts/deploy.sh` 먼저 실행 |
| `playwright test` 실패 — wrong BASE_URL | 환경변수 오류 | `playwright.config.ts` 확인, `BASE_URL` 재설정 |
| xcodebuild timeout | 시뮬레이터 기동 실패 | 시뮬레이터 재시작 후 재시도 (재시도 1회만) |
| "No such module 'Testing'" | SourceKit 노이즈 | 빌드 에러 아님, xcodebuild 결과로만 판단 |

---

## KPI 연동

| 지표 | 측정 방법 | 파일 |
|------|----------|------|
| Playwright 실행 가능 | `npx playwright test` exit 0 | `web/e2e/smoke.spec.ts` |
| UITest pass율 | xcodebuild 실행 결과 | `progress/kpi/YYMMDD_ios_uitest_status.md` |
| E2E 리포트 존재 | 파일 존재 여부 | `progress/kpi/YYMMDD_e2e_report.md` |

---

## TODO (M4 예정)

- [ ] 실제 E2E 시나리오 작성 (로그인, 피드, 온보딩)
- [ ] DB 격리 헬퍼 구현 (`e2e/helpers/db-reset.ts`)
- [ ] iOS E2ETestHelper.swift 구현
- [ ] webServer 자동화 (deploy.sh 선행 실행 자동화)
- [ ] CI 파이프라인 연동
