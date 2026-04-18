---
name: kpi
description: 현재 활성 MVP의 KPI 대시보드 출력. 지표별 목표/현재값/게이트 상태를 표로 보여주고 미달/미측정 항목을 안내한다. 트리거 키워드 KPI, 커버리지 확인, 게이트 확인, 마일스톤 지표.
allowed-tools:
  - Bash
  - Read
---

# KPI 대시보드 (/kpi)

## 수행 작업

1. `bash scripts/kpi-report.sh` 실행해 active MVP의 KPI 전체 표 조회.
2. 결과를 그대로 사용자에게 출력.
3. 미달(✗)·미측정(?) 지표가 있으면 원인과 다음 행동을 요약해 제시:
   - 측정 캐시가 없을 경우 → 해당 플랫폼 테스트·커버리지 실행 커맨드 안내
   - `manual` 지표(E2E·P50 등) → `progress/kpi/{YYMMDD}_manual.md`에 기입 방법 안내
4. 사용자가 특정 지표 세부를 요청하면 해당 플랫폼 커맨드(cargo tarpaulin·npm run coverage·ios coverage.sh)를 제안.

## 전제 조건

- `progress/active_mvp.txt`에 현재 MVP 번호가 기록되어 있어야 한다.
- 해당 MVP 기획 문서(`progress/*MVP{N}*.md` 또는 `history/mvp{N}/_roadmap.md`)에 `## KPI` 섹션이 있어야 한다.
- 선언이 없으면 `/milestone`으로 새 마일스톤을 시작하고 `progress/TEMPLATE_mvp_kpi.md` 템플릿을 따라 섹션을 추가하도록 안내한다.

## 관련 파일

- `scripts/kpi-check.sh` — pre-commit Hard 게이트 (exit 1 차단)
- `scripts/kpi-report.sh` — 대시보드(non-blocking)
- `scripts/kpi-lib.sh` — 파서·측정 공통 라이브러리
- `progress/TEMPLATE_mvp_kpi.md` — KPI 선언 표준 포맷
- `progress/kpi/` — 일별 스냅샷·수동 기록·bypass 로그
- `rules/0_CODEX_RULES.md §3.5` — 게이트 정책
