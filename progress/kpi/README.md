# KPI 디렉토리

이 디렉토리는 마일스톤 선언형 KPI 시스템의 런타임 데이터를 보관한다.

## 구조

- `{YYMMDD}.md` — 일별 KPI 스냅샷 (`scripts/kpi-report.sh` 출력)
- `{YYMMDD}_manual.md` — 수동 기록 지표 (E2E 성공률·P50 로딩 등 파서가 자동 측정 못하는 값)
- `bypass.log` — `KPI_BYPASS=1` 우회 이력 (커밋 차단 우회 시 사유 의무 기록)

## 상위 개념

- 선언: 각 MVP 기획 문서의 `## KPI` 섹션 (템플릿 `progress/TEMPLATE_mvp_kpi.md`)
- active: `progress/active_mvp.txt` 1줄 — 현재 MVP 번호
- 게이트: `.git/hooks/pre-commit` → `scripts/kpi-check.sh` → Hard 지표 미달 시 exit 1

상세: `rules/0_CODEX_RULES.md §3.5`, `CLAUDE.md` KPI 게이트 섹션.
