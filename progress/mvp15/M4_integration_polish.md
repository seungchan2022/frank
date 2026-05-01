# M4: 통합 검증 + UX 폴리싱

> 프로젝트: Frank MVP15
> 상태: 대기
> 예상 기간: 1~2주 (실사용 1주 포함)
> 의존성: M3 (핵심 기능 완료)

## 목표

M1~M3 결과를 통합 검증하고, 본인 실사용 기반으로 UX 마찰을 다듬은 후 회고를 작성한다.

## 일정 전략 (critical-review M5)

"본인 1주 실사용"이 일정에 포함되므로 **M3 후반과 dogfooding 겹치기**:
- M3 서버 핵심 (아이템 1~4) 완료 시점부터 본인이 사용 시작
- M3 클라이언트 (아이템 5~8) 완료 후 본격 1주 실사용
- M4 단독 기간은 폴리싱 + 회고 (3~5일)
- 총 M3+M4 = 약 2주

## 성공 기준 (Definition of Done)

- [ ] 웹 E2E 시나리오 추가: "프로필 설정 → 피드 새로고침 → 인사이트 표시 확인" (Playwright)
- [ ] iOS XCUITest 시나리오 추가: "직업 입력 → 피드 새로고침 → 인사이트 표시 확인"
- [ ] 본인 1주 실사용 — 발견한 마찰 기록 (`progress/mvp15/usage_notes.md`)
- [ ] 발견 마찰 중 작은 폴리싱 (1~2일 분량) 반영. 큰 변경은 부채(MVP16)로 이관
- [ ] 무료 한도 모니터링 로그 검토 — $0 유지 확인
- [ ] M1 진단 보고서 권고 중 후속 처리: Q4·Q5 결정을 MVP16 시드(`progress/mvp16/_seed.md`)에 기록
- [ ] MVP15 회고 작성 (`history/mvp15/retro.md` + html — 메모리 `feedback_daily_retro_format`)
- [ ] MVP15 폴더 → history/mvp15/ 이관

## 아이템

| # | 아이템 | 유형 | 실행 스킬 | 상태 |
|---|--------|------|----------|------|
| 1 | 웹 E2E 시나리오 추가 + 검증 | feature | /workflow | 대기 |
| 2 | iOS XCUITest 시나리오 추가 + 검증 | feature | /workflow | 대기 |
| 3 | 본인 1주 실사용 + 마찰 기록 | chore | 직접 실행 | 대기 |
| 4 | 발견 마찰 폴리싱 (작은 단위만) | feature | /workflow | 대기 |
| 5 | 무료 한도 로그 검토 + cost_log.md 정리 | chore | 직접 실행 | 대기 |
| 6 | MVP16 시드 작성 (M1 권고 흡수) | chore | 직접 실행 | 대기 |
| 7 | MVP15 회고 작성 | chore | /daily-retro 또는 직접 | 대기 |
| 8 | history 이관 + active_mvp.txt → 16:planning | chore | 직접 실행 | 대기 |

## 워크플로우 진입점

E2E 시나리오 추가는 `/workflow` 진입, 나머지는 chore라 직접 실행.

```
/workflow "M4-integration-polish"
```

## KPI (M4)

| 지표 | 측정 방법 | 목표 | 게이트 | 기준선 |
|---|---|---|---|---|
| 웹 E2E 시나리오 통과 | Playwright 실행 | 통과 | Hard | — |
| iOS XCUITest 시나리오 통과 | xcodebuild test UITests | 통과 | Hard | — |
| 본인 실사용 마찰 기록 | `progress/mvp15/usage_notes.md` | exists, ≥ 5 항목 | Hard | — |
| 무료 한도 로그 검토 결과 | `progress/mvp15/cost_log.md` | $0 유지 명시 | Hard | — |
| MVP16 시드 작성 | `progress/mvp16/_seed.md` | exists | Hard | — |
| MVP15 회고 작성 | `history/mvp15/retro.md` + `.html` | 둘 다 exists | Hard | — |
| 모든 플랫폼 테스트 통과 | 서버+웹+iOS 풀 테스트 | 통과 | Hard | MVP14 카운트 |

## 리스크

| 리스크 | 영향(H/M/L) | 대응 |
|--------|------------|------|
| 본인 실사용 1주 동안 마찰이 너무 많이 발견 | M | M4에서는 작은 폴리싱만, 큰 건 MVP16 부채로 |
| E2E 시나리오가 LLM 호출 변동성으로 불안정 | M | 인사이트 텍스트 정확 매칭 X, "단락이 비어있지 않음" 정도만 검증 |
| 회고 미작성 (KPI Hard) | H | KPI Hard라 미작성 시 커밋 차단. M3 완료 시점에 회고 시작 알람 설정 |
