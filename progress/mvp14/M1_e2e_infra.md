# M1: E2E 인프라 세팅

> 프로젝트: Frank MVP14
> 상태: 대기
> 예상 기간: 3~5일
> 의존성: 없음

## 목표

M2(버그 수정)·M3(UX 개선) 작업 중 바로 활용할 수 있도록, 웹(Playwright)과 iOS(XCUITest) E2E 실행 환경과 `/e2e` 스킬 뼈대를 먼저 구축한다. 시나리오 내용은 M4에서 작성한다.

## 성공 기준 (Definition of Done)

- [ ] 기존 iOS UITest 4개 현재 상태 확인 (pass/fail 실행 결과 기록)
- [ ] Playwright 실행 환경 구성 완료 — `npx playwright test` 한 번에 실행 가능
- [ ] XCUITest 파일 구조 정리 — 기존 4개 파일 확인 + 신규 파일 추가 위치 확정
- [ ] `/e2e` 스킬 파일 뼈대 생성 (`/.claude/skills/e2e/SKILL.md`) — 실행 흐름 명세만, 시나리오 내용 없음
- [ ] E2E 리포트 형식 확립 (`progress/kpi/YYMMDD_e2e_report.md` 포맷 + 빈 예시 1건)
- [ ] E2E 격리 전략 문서화 — 시나리오 실행 전 DB 상태 초기화 방법

## 아이템

| # | 아이템 | 유형 | 플랫폼 | 상태 |
|---|--------|------|--------|------|
| 1 | 기존 iOS UITest 4개 실행 확인 (pass/fail 기록) | research | iOS | 대기 |
| 2 | Playwright 실행 환경 구성 + 빈 테스트 파일 1개로 실행 검증 | chore | Web | 대기 |
| 3 | XCUITest 파일 구조 정리 + 신규 파일 추가 위치 확정 | chore | iOS | 대기 |
| 4 | `/e2e` 스킬 파일 뼈대 작성 | chore | — | 대기 |
| 5 | E2E 리포트 형식 확립 + 빈 예시 1건 생성 | chore | — | 대기 |
| 6 | E2E 격리 전략 문서화 (DB 초기화 방법) | chore | — | 대기 |

## 워크플로우 진입점

```
/workflow "M1-E2E 인프라: Playwright 환경 구성 + XCUITest 구조 정리 + /e2e 스킬 뼈대"
```

## KPI (M1)

| 지표 | 측정 방법 | 목표 | 게이트 |
|---|---|---|---|
| Playwright 실행 가능 | `npx playwright test` 성공 | 통과 | Hard |
| 기존 UITest 상태 기록 | xcodebuild test 실행 결과 | pass/fail 기록 완료 | Hard |
| `/e2e` 스킬 파일 존재 | `.claude/skills/e2e/SKILL.md` exists | exists | Hard |
| E2E 리포트 포맷 예시 존재 | `progress/kpi/` 파일 존재 | exists | Soft |
