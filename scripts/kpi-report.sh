#!/usr/bin/env bash
# kpi-report.sh — 2층 KPI 대시보드 (non-blocking)
#   마일스톤 KPI + (MVP completing일 때) MVP 최종 KPI 두 섹션 출력
#
# 사용: scripts/kpi-report.sh [--quick]
#   --quick: 1줄 요약만 (Stop hook용)

set -u
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/kpi-lib.sh"

QUICK=0
[[ "${1:-}" == "--quick" ]] && QUICK=1

MVP="$(kpi_active_mvp)"
MVP_STATE="$(kpi_active_state)"
MS="$(kpi_active_milestone)"
MS_STATE="$(kpi_active_milestone_state)"

if [[ -z "$MVP" ]]; then
  echo "KPI: (no active MVP)"
  exit 0
fi

# 주어진 문서 + 상태로 표를 출력하고 pass/fail/unknown/hard_fail 갱신
# 전역: pass, fail, unknown, total, hard_fail
render_section() {
  local doc="$1" eval_state="$2" heading="$3"
  local rows=""
  local TABLE
  TABLE="$(kpi_parse_table <"$doc")"
  if [[ -z "$TABLE" ]]; then
    printf "\n=== %s ===\n문서: %s\n(## KPI 섹션 없음)\n" "$heading" "${doc#"$REPO_ROOT"/}"
    return
  fi

  while IFS='|' read -r metric measure target gate baseline; do
    [[ -z "$metric" ]] && continue
    total=$((total+1))
    local key current
    key="$(kpi_measure_key "$metric")"
    current=""
    case "$key" in
      server_cov)    current="$(kpi_measure_server_cov)" ;;
      web_cov)       current="$(kpi_measure_web_cov)" ;;
      ios_cov)       current="$(kpi_measure_ios_cov)" ;;
      retro_exists)  current="$(kpi_measure_retro_exists "$MVP")" ;;
      debt_count)    current="$(kpi_measure_debt_count)" ;;
      *)             current="" ;;
    esac
    [[ -z "$current" ]] && current="-"

    local effective_gate gate_label
    effective_gate="$(kpi_gate_resolve "$eval_state" "$gate" "$key" "$current")"
    gate_label="$gate"
    [[ "$gate" != "$effective_gate" ]] && gate_label="${gate}→${effective_gate}"

    kpi_eval_target "$target" "$current"
    local rc=$?
    local status
    case $rc in
      0) status="✓"; pass=$((pass+1)) ;;
      1) status="✗"; fail=$((fail+1)); [[ "$effective_gate" == "Hard" ]] && hard_fail=$((hard_fail+1)) ;;
      2) status="?"; unknown=$((unknown+1)); [[ "$effective_gate" == "Hard" ]] && hard_fail=$((hard_fail+1)) ;;
    esac

    rows+="$(printf '%-34s %-14s %-12s %-12s %s' "$metric" "$gate_label" "$target" "$current" "$status")"$'\n'
  done <<<"$TABLE"

  printf "\n=== %s ===\n" "$heading"
  printf "문서: %s\n\n" "${doc#"$REPO_ROOT"/}"
  printf "%-34s %-14s %-12s %-12s %s\n" "지표" "Gate(선언→적용)" "Target" "Current" "Status"
  printf "%-34s %-14s %-12s %-12s %s\n" "----" "--------------" "------" "-------" "------"
  echo -n "$rows"
}

pass=0; fail=0; unknown=0; total=0; hard_fail=0

# 마일스톤 섹션 (항상 시도)
MS_DOC=""
if [[ -n "$MS" ]]; then
  MS_DOC="$(kpi_find_milestone_doc "$MVP" "$MS")" || true
fi

# MVP 최종 섹션 (MVP completing일 때만 표시)
FINAL_DOC=""
if [[ "$MVP_STATE" == "completing" ]]; then
  FINAL_DOC="$(kpi_find_mvp_final_doc "$MVP")" || true
fi

# QUICK 모드 — 요약만
if [[ $QUICK -eq 1 ]]; then
  # 값 계산 위해 호출
  if [[ -n "${MS_DOC:-}" ]]; then
    render_section "$MS_DOC" "$MS_STATE" "마일스톤" >/dev/null
  fi
  if [[ -n "${FINAL_DOC:-}" ]]; then
    render_section "$FINAL_DOC" "completing" "MVP 최종" >/dev/null
  fi
  echo "KPI MVP${MVP}[${MVP_STATE}]${MS:+ / $MS[$MS_STATE]}: ✓${pass} ✗${fail} ?${unknown} / ${total} (Hard미달 ${hard_fail})"
  exit 0
fi

# 전체 출력
if [[ -n "${MS_DOC:-}" ]]; then
  render_section "$MS_DOC" "$MS_STATE" "마일스톤 KPI — MVP${MVP} / ${MS} [${MS_STATE}]"
elif [[ -n "$MS" ]]; then
  printf "\n=== 마일스톤 KPI — MVP%s / %s ===\n" "$MVP" "$MS"
  printf "문서: (없음) — progress/mvp%s/%s_*.md 를 만들고 ## KPI 섹션을 추가하세요\n" "$MVP" "$MS"
fi

if [[ -n "${FINAL_DOC:-}" ]]; then
  render_section "$FINAL_DOC" "completing" "MVP 최종 KPI — MVP${MVP} [completing]"
elif [[ "$MVP_STATE" == "completing" ]]; then
  printf "\n=== MVP 최종 KPI ===\n(문서 없음 — progress/mvp%s/_roadmap.md 등에 ## KPI 섹션 추가)\n" "$MVP"
fi

# 마일스톤 문서도 없고 MVP completing도 아니면 MVP 기획 폴백
if [[ -z "${MS_DOC:-}" && -z "${FINAL_DOC:-}" ]]; then
  FB="$(kpi_find_plan_doc "$MVP")" || true
  if [[ -n "${FB:-}" ]]; then
    render_section "$FB" "$MVP_STATE" "MVP 기획 KPI — MVP${MVP} [${MVP_STATE}] (폴백)"
  fi
fi

printf "\n결과: ✓%d 통과  ✗%d 미달  ?%d 미측정 (총 %d)  Hard미달 %d\n" "$pass" "$fail" "$unknown" "$total" "$hard_fail"
exit 0
