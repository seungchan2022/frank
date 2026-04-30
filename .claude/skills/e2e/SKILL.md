# /e2e 스킬

> 상태: M4 시나리오 완성 (웹 W-01~W-03, iOS I-01~I-05).
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

## 시나리오 목록 (M4)

### 웹 W-01~W-03

| ID | 파일 | 커버 항목 | 테스트 수 |
|----|------|-----------|----------|
| W-01 | `web/e2e/feed-summary.spec.ts` | BUG-006(에러 재시도), DEBT-06(하단 고정), DEBT-07(카드 구분) | 4개 |
| W-02 | `web/e2e/tag-navigation.spec.ts` | BUG-008(탭 전환 깜빡임) | 2개 |
| W-03 | `web/e2e/feed-like.spec.ts` | DEBT-04(좋아요 단독 탭, URL 유지) | 3개 |

### iOS I-01~I-05

| ID | 파일 | 커버 항목 |
|----|------|-----------|
| I-01 | `LoginFlowUITest.swift` | 로그인 → 피드 진입 + 기사 로딩 확인 |
| I-02 | `CrossFeatureFlowUITest.swift::testTagTabSwitching` | BUG-008 탭 전환 피드 유지 |
| I-03 | `M3UXImprovementsUITest.swift::testDetailSummaryThenActionButton` | BUG-006+DEBT-06 요약 후 버튼 접근 |
| I-04 | `FeedRefreshUITest.swift` | BUG-007 pull-to-refresh (2단계 Mock) |
| I-05 | `M3UXImprovementsUITest.swift::testFeedLikeButtonDoesNotNavigateToDetail` | DEBT-04 좋아요 단독 탭 |

---

## 파일 구조

### 웹 E2E

```
web/
├── playwright.config.ts          — Playwright 설정 (BASE_URL, chromium)
└── e2e/
    ├── smoke.spec.ts             — smoke 테스트 (인프라 검증용)
    ├── feed-summary.spec.ts      — W-01: 요약 + DEBT-06/07
    ├── tag-navigation.spec.ts    — W-02: 태그 탭 전환 + BUG-008
    └── feed-like.spec.ts         — W-03: 좋아요 단독 탭 + DEBT-04
```

### iOS E2E

```
ios/Frank/FrankUITests/
├── LoginFlowUITest.swift         — TC-01 + I-01: 로그인 → 피드 + 기사 로딩
├── CrossFeatureFlowUITest.swift  — TC-04 + I-02: 크로스 피처 + 태그 탭 전환
├── M3UXImprovementsUITest.swift  — I-03/I-05: 요약 후 버튼 + 좋아요 단독 탭
├── FeedRefreshUITest.swift       — I-04: pull-to-refresh (신규)
└── OnboardingFlowUITest.swift    — TC-02: 신규 유저 온보딩
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

## 신규 시나리오 done 처리 전 — 하네스 검증 체크리스트 (필수)

**원칙**: 자동화는 *통과*만으론 신뢰할 수 없다. 시나리오 자체가 의도한 사용성을 검증하는지 사용자가 직접 매칭 1회 확인 후에야 마일스톤/PR을 닫는다. (`feedback_harness_verification`)

신규 E2E 시나리오 추가 후 done 처리 직전, 사용자에게 다음을 명시 확인:

- [ ] 시나리오가 통과한다 (`xcodebuild test` / `npx playwright test` exit 0)
- [ ] **사용자가 동일 시나리오를 시뮬레이터/브라우저에서 직접 1회 재현해 봤다**
- [ ] 직접 재현 결과와 자동 통과 결과가 일치한다 (단순 통과가 아니라 "의도한 동작이 정말 발생했는가" 매칭)
- [ ] 매칭이 어긋나면 시나리오를 수정한다 — 이후에야 done

**적용 시점**: 마일스톤 시나리오 신규 추가 / 기존 시나리오 큰 폭 변경 / Mock 분기 신규 도입 시. 단순 리팩토링·셀렉터 변경은 면제.

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

## 완료 (M4)

- [x] 웹 E2E 시나리오 W-01~W-03 작성 완료 (260430)
- [x] iOS UITest I-01~I-05 시나리오 작성/확장 완료 (260430)
- [x] FeedRefreshUITest.swift 신규 파일 + MockArticleAdapter 2단계 지원 (260430)
- [x] W-02 MutationObserver 순서 버그 수정 — observer 클릭 전 전역 설치 + expect 어설션 (260430)
- [x] I-04 retry 로직 추가 — app.tables 우선 접근 + 최대 2회 swipeDown retry (260430)
- [x] E2E 리포트 생성 — `progress/kpi/260430_e2e_report.md` (260430)

## TODO (다음 MVP)

- [ ] DB 격리 헬퍼 구현 (`e2e/helpers/db-reset.ts`) — 세션 독립적 실행 지원
- [ ] webServer 자동화 (deploy.sh 선행 실행 자동화)
- [ ] CI 파이프라인 연동
