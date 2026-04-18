#!/usr/bin/env bash
# kpi-check.sh — 2층 KPI 게이트 검증
#
# 검증 대상 결정 규칙:
#   1) MVP 상태가 "completing"이면 → MVP 최종 KPI 검증 (_roadmap.md / _vision.md)
#   2) 그 외 (planning/in-progress) → 현재 active 마일스톤의 KPI 검증 (M{X}_*.md)
#   3) active 마일스톤이 없거나 문서가 없으면 → MVP 기획 문서 폴백
#
# exit code:
#   0 — 적용된 Hard 지표 전부 통과 또는 선언된 Hard 없음
#   1 — Hard 지표 미달 발견 (커밋 차단)

set -u
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/kpi-lib.sh"

BYPASS="${KPI_BYPASS:-0}"
if [[ "$BYPASS" == "1" ]]; then
  mkdir -p "$REPO_ROOT/progress/kpi"
  echo "$(date +%Y-%m-%dT%H:%M:%S) BYPASS by $USER" >>"$REPO_ROOT/progress/kpi/bypass.log"
  echo "[kpi-check] KPI_BYPASS=1 — 검증 스킵 (사유를 progress/kpi/bypass.log에 꼭 기록하세요)"
  exit 0
fi

MVP="$(kpi_active_mvp)"
MVP_STATE="$(kpi_active_state)"
if [[ -z "$MVP" ]]; then
  echo "[kpi-check] progress/active_mvp.txt 없음 → 게이트 스킵"
  exit 0
fi
if [[ "$MVP_STATE" == "done" ]]; then
  echo "[kpi-check] MVP${MVP} 상태 = done → 다음 MVP로 active_mvp.txt 갱신 필요. 게이트 스킵"
  exit 0
fi

MS="$(kpi_active_milestone)"
MS_STATE="$(kpi_active_milestone_state)"

# 검증 대상 문서 + 검증에 쓸 상태 결정
DOC=""
SCOPE=""       # "milestone" | "mvp-final" | "mvp-plan"
EVAL_STATE=""  # kpi_gate_resolve에 넘길 상태

if [[ "$MVP_STATE" == "completing" ]]; then
  # MVP 최종 검증
  DOC="$(kpi_find_mvp_final_doc "$MVP")" || true
  SCOPE="mvp-final"
  EVAL_STATE="completing"
else
  # 마일스톤 단위 검증 (planning/in-progress 공통)
  if [[ -n "$MS" ]]; then
    DOC="$(kpi_find_milestone_doc "$MVP" "$MS")" || true
    SCOPE="milestone"
    EVAL_STATE="$MS_STATE"
  fi
  if [[ -z "${DOC:-}" ]]; then
    DOC="$(kpi_find_plan_doc "$MVP")" || true
    SCOPE="mvp-plan"
    EVAL_STATE="$MVP_STATE"
  fi
fi

if [[ -z "${DOC:-}" ]]; then
  echo "[kpi-check] MVP${MVP} ${MS:+/ $MS} 검증 대상 문서 없음 → 게이트 스킵"
  echo "  (progress/TEMPLATE_mvp_kpi.md 참고해 해당 문서에 ## KPI 섹션 추가)"
  exit 0
fi

TABLE="$(kpi_parse_table <"$DOC")"
if [[ -z "$TABLE" ]]; then
  echo "[kpi-check] ${DOC#"$REPO_ROOT"/}에 ## KPI 섹션 없음 → 게이트 스킵"
  exit 0
fi

# 헤더
case "$SCOPE" in
  milestone)  HEADER="MVP${MVP} / ${MS} [${MS_STATE}] (마일스톤 KPI)" ;;
  mvp-final)  HEADER="MVP${MVP} [completing] (MVP 최종 KPI)" ;;
  mvp-plan)   HEADER="MVP${MVP} [${MVP_STATE}] (MVP 기획 KPI — 마일스톤 문서 없음 폴백)" ;;
esac

fail=0
printf "\n=== KPI check — %s ===\n" "$HEADER"
printf "문서: %s\n\n" "${DOC#"$REPO_ROOT"/}"
printf "%-34s %-14s %-12s %-12s %s\n" "지표" "Gate(선언→적용)" "Target" "Current" "Status"
printf "%-34s %-14s %-12s %-12s %s\n" "----" "--------------" "------" "-------" "------"

while IFS='|' read -r metric measure target gate baseline; do
  [[ -z "$metric" ]] && continue
  key="$(kpi_measure_key "$metric")"
  current=""
  case "$key" in
    server_cov)    current="$(kpi_measure_server_cov)" ;;
    web_cov)       current="$(kpi_measure_web_cov)" ;;
    ios_cov)       current="$(kpi_measure_ios_cov)" ;;
    retro_exists)  current="$(kpi_measure_retro_exists "$MVP")" ;;
    debt_count)    current="$(kpi_measure_debt_count)" ;;
    manual|*)      manual_file="$REPO_ROOT/progress/kpi/$(date +%Y%m%d)_manual.md"
                   if [[ -f "$manual_file" ]]; then
                     current="$(grep -F "$metric" "$manual_file" | head -1 | awk -F: '{print $NF}' | tr -d '[:space:]')"
                   fi
                   ;;
  esac
  [[ -z "$current" ]] && current="-"

  effective_gate="$(kpi_gate_resolve "$EVAL_STATE" "$gate" "$key" "$current")"
  gate_label="$gate"
  [[ "$gate" != "$effective_gate" ]] && gate_label="${gate}→${effective_gate}"

  kpi_eval_target "$target" "$current"
  case $? in
    0) status="✓" ;;
    1) status="✗"; [[ "$effective_gate" == "Hard" ]] && fail=$((fail+1)) ;;
    2) status="?"; [[ "$effective_gate" == "Hard" ]] && fail=$((fail+1)) ;;
  esac

  printf "%-34s %-14s %-12s %-12s %s\n" "$metric" "$gate_label" "$target" "$current" "$status"
done <<<"$TABLE"

echo ""
if [[ $fail -gt 0 ]]; then
  echo "[kpi-check] FAIL: Hard 지표 ${fail}개 미달/미측정 — 커밋 차단"
  echo "  * 측정 캐시 갱신: cargo tarpaulin / npm run coverage / ios coverage.sh"
  echo "  * 수동 지표: progress/kpi/$(date +%Y%m%d)_manual.md 에 \"지표명: 값\" 형식으로 기록"
  echo "  * 긴급 우회: KPI_BYPASS=1 git commit ... (사유는 progress/kpi/bypass.log에 기록됨)"
  exit 1
fi

echo "[kpi-check] PASS"
exit 0
