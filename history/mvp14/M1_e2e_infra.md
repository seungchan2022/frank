# M1: E2E 인프라 세팅

> 프로젝트: Frank MVP14
> 상태: 완료 (2026-04-29)
> 예상 기간: 3~5일
> 의존성: 없음

## 목표

M2(버그 수정)·M3(UX 개선) 작업 중 바로 활용할 수 있도록, 웹(Playwright)과 iOS(XCUITest) E2E 실행 환경과 `/e2e` 스킬 뼈대를 먼저 구축한다. 시나리오 내용은 M4에서 작성한다.

## 성공 기준 (Definition of Done)

- [x] 기존 iOS UITest 4개 현재 상태 확인 (pass/fail 실행 결과 기록) — 4/4 PASS
- [x] Playwright 실행 환경 구성 완료 — `npx playwright test` 한 번에 실행 가능
- [x] XCUITest 파일 구조 정리 — 기존 3개 파일 확인 + 신규 파일 추가 위치 확정 (Tuist glob 커버 확인)
- [x] `/e2e` 스킬 파일 뼈대 생성 (`.claude/skills/e2e/SKILL.md`) — 실행 흐름 명세만, 시나리오 내용 없음
- [x] E2E 리포트 형식 확립 (`progress/kpi/260429_e2e_report_example.md` 포맷 + 빈 예시 1건)
- [x] E2E 격리 전략 문서화 — `progress/docs/e2e_isolation_strategy.md` 작성 완료

## 아이템

| # | 아이템 | 유형 | 플랫폼 | 상태 |
|---|--------|------|--------|------|
| 1 | 기존 iOS UITest 4개 실행 확인 (pass/fail 기록) | research | iOS | ✅ done |
| 2 | Playwright 실행 환경 구성 + 빈 테스트 파일 1개로 실행 검증 | chore | Web | ✅ done |
| 3 | XCUITest 파일 구조 정리 + 신규 파일 추가 위치 확정 | chore | iOS | ✅ done |
| 4 | `/e2e` 스킬 파일 뼈대 작성 | chore | — | ✅ done |
| 5 | E2E 리포트 형식 확립 + 빈 예시 1건 생성 | chore | — | ✅ done |
| 6 | E2E 격리 전략 문서화 (DB 초기화 방법) | chore | — | ✅ done |

## 서브태스크 분해 (Step 3)

> 분해 기준: 단일 책임 · 독립 실행 · 명확한 산출물 · 1~2시간 내 완료

| ID | 서브태스크 | 플랫폼 | 산출물 | 의존성 | 실행 Phase |
|----|-----------|--------|--------|--------|-----------|
| ST-1 | iOS UITest 현재 상태 확인 및 기록 | iOS | `progress/kpi/260429_ios_uitest_status.md` (pass/fail 기록) | 없음 | Phase 1 |
| ST-2 | Playwright 실행 환경 구성 및 실행 검증 | Web | `web/playwright.config.ts`, `web/e2e/smoke.spec.ts`, package.json 업데이트, `npx playwright test` 성공 | 없음 | Phase 1 |
| ST-3 | XCUITest 파일 구조 정리 | iOS | 신규 파일 추가 위치 확정 + Tuist Project.swift 확인/수정 + 구조 명세 | ST-1 참고 (독립 가능) | Phase 2 |
| ST-4 | E2E 격리 전략 문서화 | 서버 | `progress/docs/e2e_isolation_strategy.md` (DB 초기화 방법 + 격리 전략) | 없음 | Phase 1 |
| ST-5 | /e2e 스킬 파일 뼈대 + E2E 리포트 포맷 생성 | 공통 | `.claude/skills/e2e/SKILL.md`, `progress/kpi/260429_e2e_report_example.md` | ST-2, ST-3, ST-4 | Phase 3 |

### 실행 순서

```
Phase 1 (병렬): ST-1 + ST-2 + ST-4
Phase 2 (ST-1 완료 후): ST-3
Phase 3 (Phase 1+2 완료 후): ST-5
```

### 의존성 DAG

```
ST-1 ──┐
ST-2 ──┼──► ST-5
ST-3 ──┤
ST-4 ──┘
```

ST-1 → ST-3 (참고 관계, ST-3은 ST-1 결과 확인 후 구조 확정)

## 워크플로우 진입점

```
/workflow "M1-E2E 인프라: Playwright 환경 구성 + XCUITest 구조 정리 + /e2e 스킬 뼈대"
```

## 인터뷰 결정사항 (Step 4)

| 항목 | 결정 | 비고 |
|------|------|------|
| iOS UITest 확인 방식 | xcodebuild test 전체 실행 (iPhone 17 Pro) | pass/fail 기준선 확보 |
| Playwright baseURL | 환경변수 BASE_URL (default: localhost:5173) | — |
| E2E 격리 전략 | 문서화만 (실제 구현은 M4) | — |
| /e2e 스킬 범위 | 실행 흐름 + 플랫폼별 명령어 | 시나리오 없음 |
| Playwright 백엔드 | local real backend (deploy.sh 선행 실행) | webServer 옵션 비활성화 |
| deploy.sh 자동화 | webServer 비활성화 + SKILL.md에 선행 실행 명시 | 자동화는 M4 부채로 이관 |
| smoke 시나리오 | 더미 테스트만 (시나리오는 M4) | — |

## Feature List
<!-- size: 대형 | count: 28 | skip: false -->

### 기능
- [x] F-01 web/package.json에 @playwright/test devDependency 추가
- [x] F-02 web/playwright.config.ts 생성 — BASE_URL 환경변수 방식 적용
- [x] F-03 web/e2e/smoke.spec.ts 더미 테스트 1개 생성
- [x] F-04 npx playwright test 실행 시 exit 0 확인
- [x] F-05 xcodebuild test (iPhone 17 Pro) 실행 — UITest 4개 pass/fail 기록
- [x] F-06 progress/kpi/260429_ios_uitest_status.md 결과 파일 생성
- [x] F-07 AuthFlowUITest.swift 위치 이슈 기록 — FrankTests 유지 결정 (XCUITest 아님)
- [x] F-08 XCUITest 신규 파일 추가 위치 및 네이밍 규칙 확정
- [x] F-09 .claude/skills/e2e/SKILL.md 뼈대 생성 — 실행 흐름 + 명령어 명세
- [x] F-10 progress/kpi/260429_e2e_report_example.md 빈 예시 1건 생성
- [x] F-11 progress/docs/e2e_isolation_strategy.md — DB 초기화 전략 문서화

### 엣지
- [x] E-01 BASE_URL 환경변수 미설정 시 localhost:5173 fallback 동작 확인
- [-] N/A (smoke.spec.ts는 about:blank 사용 — 실서버 연결 시나리오는 M4) E-02 서버 미기동 상태에서 npx playwright test 실행 시 에러 메시지 명확성 확인
- [x] E-03 AuthFlowUITest.swift가 FrankTests 타겟에서 실행될 때 XCUITest API 접근 가능 여부 — XCUITest 미사용 확인, FrankTests 유지
- [x] E-04 Playwright 설치 후 기존 vitest 테스트(`npm run test`) 영향 없음 확인 — 265/265 통과
- [~] deferred (시뮬레이터 실패 케이스는 M4에서 실제 UITest 실행 시 경험적 확인) E-05 xcodebuild test 중 시뮬레이터 기동 실패 시 재시도 없이 fail 기록

### 에러
- [x] R-01 npx playwright test 실행 실패 시 에러 로그 수집 방법 명세 (SKILL.md)
- [x] R-02 xcodebuild test 타임아웃 발생 시 처리 방법 문서화 (SKILL.md)
- [~] deferred (실서버 연결 시나리오는 M4 이후) R-03 playwright.config.ts baseURL 잘못된 값 입력 시 spec 파일 에러 메시지 확인

### 테스트
- [x] T-01 npm run test:e2e 명령 실행 → smoke.spec.ts 통과
- [x] T-02 npm run test (Vitest) — 기존 테스트 전부 통과 (회귀 없음) — 265/265 pass
- [x] T-03 xcodebuild test -scheme Frank — UITest 4개 실행 결과 기록 (4/4 PASS)
- [x] T-04 .claude/skills/e2e/SKILL.md 내 명령어 복붙 시 실제 실행 가능 여부 수동 확인

### 플랫폼
- [x] P-01 tuist generate 선행 후 xcodebuild test 실행 가능 확인
- [x] P-02 FrankUITests 타겟에 신규 파일 추가 시 Tuist Project.swift 수정 불필요 확인 — `FrankUITests/**` glob 커버 확인
- [x] P-03 AuthFlowUITest.swift — FrankTests 유지 결정 (Mock 기반 통합 테스트, XCUITest 아님)

### 회귀
- [x] G-01 Playwright 설치로 인한 web/ 빌드 시간 증가 여부 측정 — 2.3초, 회귀 없음
- [x] G-02 package.json 변경 후 npm run check (SvelteKit 타입체크) 통과 확인

## KPI (M1)

| 지표 | 측정 방법 | 목표 | 게이트 |
|---|---|---|---|
| Playwright 실행 가능 | `npx playwright test` 성공 | 통과 | Hard |
| 기존 UITest 상태 기록 | xcodebuild test 실행 결과 | pass/fail 기록 완료 | Hard |
| `/e2e` 스킬 파일 존재 | `.claude/skills/e2e/SKILL.md` exists | exists | Hard |
| E2E 리포트 포맷 예시 존재 | `progress/kpi/` 파일 존재 | exists | Soft |
