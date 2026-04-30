# 메인태스크: M4-E2E 시나리오 완성 + 검증

> 작성일: 260430
> 마일스톤: MVP14 M4
> 상태: in-progress
> 의존성: M2 (버그 수정 완료), M3 (UX 개선 완료)

## 목표

M1에서 구축한 E2E 인프라 위에, M2+M3에서 수정/개선한 항목을 기준으로
웹(Playwright W-01~W-03) + iOS(XCUITest I-01~I-05) 시나리오를 작성하고 검증한다.
`/e2e` 스킬 시나리오 내용을 채우고 실행 리포트를 생성한다.

## 인터뷰 결정사항

| 항목 | 결정 | 비고 |
|------|------|------|
| E2E 실행 방식 | 수동 기동 유지 (deploy.sh 선행) | step-8 안내 텍스트만 추가, webServer 자동화 없음 |
| iOS UITest 범위 | 기존 4개 파일 업데이트 + I-04·I-05 신규 파일 추가 | 필요한 시나리오 전체 커버 |
| E2E 실행 타이밍 | step-8에 웹+iOS 병렬 통합 | pre-commit 아님, 명시적 실행 |

## 시나리오 매핑

### 웹 Playwright

| ID | 파일 | 커버 항목 |
|----|------|----------|
| W-01 | `web/e2e/feed-summary.spec.ts` | BUG-006 (에러 캐시 재시도), DEBT-06 (요약 버튼 하단), DEBT-07 (카드 구분) |
| W-02 | `web/e2e/tag-navigation.spec.ts` | BUG-008 (탭 전환 깜빡임) |
| W-03 | `web/e2e/feed-like.spec.ts` | DEBT-04 (좋아요 단독 탭) |

### iOS XCUITest

| ID | 파일 | 커버 항목 | 방식 |
|----|------|----------|------|
| I-01 | `LoginFlowUITest.swift` | 로그인 → 피드 진입 + 기사 로딩 | 기존 확장 |
| I-02 | `CrossFeatureFlowUITest.swift` | BUG-008 (탭 전환), DEBT-05 (스와이프 탐색) | 기존 확장 (신규 메서드) |
| I-03 | `M3UXImprovementsUITest.swift` | BUG-006 (요약 후 동작), DEBT-06 (하단 버튼) | 기존 확장 (요약 후 케이스 추가) |
| I-04 | `FeedRefreshUITest.swift` | BUG-007 (pull-to-refresh) | 신규 파일 |
| I-05 | `M3UXImprovementsUITest.swift` | DEBT-04 (좋아요 단독 탭) | 기존 `testFeedLikeButtonDoesNotNavigateToDetail` → I-05 ID 주석 명시 |

## 서브태스크

| ID | 서브태스크 | 플랫폼 | 산출물 | 의존성 | Phase |
|----|-----------|--------|--------|--------|-------|
| ST-01 | W-01 시나리오 작성 (로그인→피드→요약→재시도) | Web | `web/e2e/feed-summary.spec.ts` | 없음 | 1 |
| ST-02 | W-02 시나리오 작성 (태그 탭 전환) | Web | `web/e2e/tag-navigation.spec.ts` | 없음 | 1 |
| ST-03 | W-03 시나리오 작성 (좋아요 단독 탭) | Web | `web/e2e/feed-like.spec.ts` | 없음 | 1 |
| ST-04 | LoginFlowUITest 확장 (I-01: 피드 기사 로딩 확인 추가) | iOS | `LoginFlowUITest.swift` | 없음 | 1 |
| ST-05 | CrossFeatureFlowUITest 확장 (I-02: 태그 탭 전환 + 스와이프) | iOS | `CrossFeatureFlowUITest.swift` | 없음 | 1 |
| ST-06 | M3UXImprovementsUITest 확장 (I-03: 요약 후 버튼 접근, I-05 ID 주석) | iOS | `M3UXImprovementsUITest.swift` | 없음 | 1 |
| ST-07 | FeedRefreshUITest 신규 파일 작성 (I-04: pull-to-refresh) | iOS | `FeedRefreshUITest.swift` | 없음 | 1 |
| ST-08 | E2E 실행 검증 (웹 W-01~W-03 + iOS I-01~I-05) | 공통 | 실행 결과 로그 | ST-01~07 | 2 |
| ST-09 | /e2e 스킬 시나리오 내용 채우기 | 공통 | `.claude/skills/e2e/SKILL.md` | ST-01~07 | 3 |
| ST-10 | E2E 실행 리포트 생성 | 공통 | `progress/kpi/260430_e2e_report.md` | ST-08 | 3 |

### 실행 순서

```
Phase 1 (전체 병렬): ST-01, ST-02, ST-03, ST-04, ST-05, ST-06, ST-07
Phase 2 (Phase 1 완료 후): ST-08 (E2E 실행 검증 — 웹+iOS 병렬)
Phase 3 (Phase 2 완료 후): ST-09 (스킬 업데이트), ST-10 (리포트 생성)
```

### 의존성 DAG

```
ST-01 ──┐
ST-02 ──┤
ST-03 ──┤
ST-04 ──┼──► ST-08 ──► ST-09
ST-05 ──┤           └──► ST-10
ST-06 ──┤
ST-07 ──┘
```

## Feature List

### 기능 (F)
- [ ] F-01 `web/e2e/feed-summary.spec.ts` — W-01: 로그인 → 피드 → 기사 클릭 → 요약 요청 → 요약 카드 표시 확인
- [ ] F-02 `web/e2e/feed-summary.spec.ts` — W-01: 요약 재시도 (요약 버튼 재탭 → 성공) (BUG-006)
- [ ] F-03 `web/e2e/feed-summary.spec.ts` — W-01: 요약 버튼 하단 고정 — 요약 전후 버튼 노출 확인 (DEBT-06)
- [ ] F-04 `web/e2e/feed-summary.spec.ts` — W-01: 기사 소개 카드 vs AI 요약 카드 구분 (DEBT-07) — `.card-snippet` / `.card-summary` 클래스 어설션
- [ ] F-05 `web/e2e/tag-navigation.spec.ts` — W-02: 로그인 → 태그 클릭 → 목록 변경 확인
- [ ] F-06 `web/e2e/tag-navigation.spec.ts` — W-02: 탭 전환 중 깜빡임 없음 확인 (BUG-008) — DOM 삭제 후 재삽입 없음
- [ ] F-07 `web/e2e/feed-like.spec.ts` — W-03: 로그인 → 피드 → 좋아요 버튼 클릭 → URL 변경 없음 확인 (DEBT-04)
- [ ] F-08 `web/e2e/feed-like.spec.ts` — W-03: 좋아요 후 즐겨찾기 탭에서 확인
- [ ] F-09 `LoginFlowUITest.swift` — I-01: 기존 로그인 플로우에 피드 기사 로딩 확인 추가 (cells.firstMatch 존재)
- [ ] F-10 `CrossFeatureFlowUITest.swift` — I-02: testTagTabSwitching — 태그 탭 전환 → 깜빡임 없음 (BUG-008)
- [ ] F-11 `CrossFeatureFlowUITest.swift` — I-02: 태그 스와이프 탐색 → 스와이프 제스처로 태그 이동 (DEBT-05)
- [ ] F-12 `M3UXImprovementsUITest.swift` — I-03: testDetailSummaryThenActionButton — 요약하기 탭 → 요약 진행 → 버튼 여전히 접근 가능 (BUG-006)
- [ ] F-13 `M3UXImprovementsUITest.swift` — I-05: testFeedLikeButtonDoesNotNavigateToDetail에 "// I-05" 시나리오 ID 주석 추가
- [ ] F-14 `FeedRefreshUITest.swift` — I-04: pull-to-refresh → RefreshControl 트리거 → 기사 목록 갱신 (BUG-007)
- [ ] F-15 `.claude/skills/e2e/SKILL.md` — W-01~W-03, I-01~I-05 시나리오 설명 추가
- [ ] F-16 `progress/kpi/260430_e2e_report.md` — 실행 결과 리포트 생성

### 엣지케이스 (E)
- [ ] E-01 W-01: 요약 API 실패 시 에러 메시지 노출 확인 (재시도 버튼 표시)
- [ ] E-02 W-02: 태그 선택 후 빈 목록일 때 빈 상태 UI 노출
- [ ] E-03 I-04: pull-to-refresh 완료 후 새로고침 인디케이터 사라짐 확인

### 에러 케이스 (R)
- [ ] R-01 W-01: 로그인 세션 만료 시 피드 접근 → 로그인 화면 리다이렉트 확인
- [ ] R-02 I-04: Mock 모드에서 RefreshControl 동작 확인 (네트워크 없이)

### 테스트 (T)
- [ ] T-01 `npm run test:e2e` → W-01, W-02, W-03 3개 통과
- [ ] T-02 `xcodebuild test -only-testing:FrankUITests` → I-01~I-05 커버 5개 클래스 전체 통과
- [ ] T-03 `npm run test` (Vitest) 기존 265개 회귀 없음
- [ ] T-04 `cargo test` 기존 328개 회귀 없음

### 플랫폼 (P)
- [ ] P-01 웹 E2E: `BASE_URL=http://localhost:5173 npx playwright test` 실행 성공
- [ ] P-02 iOS E2E: `tuist generate --no-open` 선행 후 `FrankUITests` 타겟 실행 성공
- [ ] P-03 신규 `FeedRefreshUITest.swift`가 Tuist `FrankUITests/**` glob으로 자동 포함 확인

### 회귀 (G)
- [ ] G-01 새 Playwright 스펙 추가 후 기존 `smoke.spec.ts` 통과 유지
- [ ] G-02 iOS UITest 확장/신규 추가 후 기존 4개 테스트 회귀 없음

## 수정 항목 ↔ 시나리오 커버리지 최종 매핑

| 수정 항목 | 커버 시나리오 | 상태 |
|-----------|-------------|------|
| BUG-006: 에러 캐시 재시도 | W-01 (재시도), I-03 (요약 후 버튼) | ⬜ 작성 필요 |
| BUG-007: pull-to-refresh | I-04 | ⬜ 작성 필요 |
| BUG-008: 탭 전환 깜빡임 | W-02, I-02 | ⬜ 작성 필요 |
| BUG-009: 썸네일 | — | N/A (외부 의존 한계) |
| DEBT-04: 좋아요 터치 | W-03, I-05 | ⬜ 작성 필요 |
| DEBT-05: 태그 스와이프 | I-02 (스와이프 탐색) | ⬜ 작성 필요 |
| DEBT-06: 요약 하단 분리 | W-01 (버튼 접근), I-03 (하단 고정) | ⬜ 작성 필요 |
| DEBT-07: 카드 UI 구분 | W-01 (클래스 어설션), I-03 (스크린샷) | ⬜ 작성 필요 |

## 산출물 목록

| 파일 | 유형 |
|------|------|
| `web/e2e/feed-summary.spec.ts` | 신규 |
| `web/e2e/tag-navigation.spec.ts` | 신규 |
| `web/e2e/feed-like.spec.ts` | 신규 |
| `ios/Frank/FrankUITests/FeedRefreshUITest.swift` | 신규 |
| `ios/Frank/FrankUITests/LoginFlowUITest.swift` | 확장 |
| `ios/Frank/FrankUITests/CrossFeatureFlowUITest.swift` | 확장 |
| `ios/Frank/FrankUITests/M3UXImprovementsUITest.swift` | 확장 |
| `.claude/skills/e2e/SKILL.md` | 업데이트 |
| `progress/kpi/260430_e2e_report.md` | 신규 |

## KPI (M4)

| 지표 | 측정 방법 | 목표 | 게이트 |
|---|---|---|---|
| 웹 E2E W-01~W-03 통과 | `npx playwright test` | 3개 통과 | Hard |
| iOS XCUITest I-01~I-05 통과 | `xcodebuild test -only-testing:FrankUITests` | 5개 클래스 통과 | Hard |
| E2E 리포트 생성 | `progress/kpi/260430_e2e_report.md` 존재 | exists | Hard |
| 기존 단위/통합 테스트 회귀 없음 | cargo test + vitest | 전체 통과 | Hard |

## 리스크

| 리스크 | 영향 | 대응 |
|--------|------|------|
| Playwright 로그인 세션 — 실 서버 연결 필요 | M | step-8 안내에 deploy.sh 선행 실행 명시, 세션 재사용 전략 |
| iOS XCUITest 스와이프 제스처(I-02) Simulator 불안정 | M | 결정론적 탭 기반 대체 방법 준비, 실패 시 skip 표시 |
| Mock 모드에서 pull-to-refresh(I-04) 트리거 감지 어려움 | M | RefreshControl 존재 확인 + 트리거 후 로딩 인디케이터 변화 어설션 |
| 웹 W-01 요약 재시도 — 실 API 필요 (캐시 키 초기화) | H | 로그인 후 캐시 초기화 API 호출 + 새 기사로 요약 요청 |
