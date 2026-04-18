#!/usr/bin/env bash
# kpi-lib.sh — 공통 KPI 파싱/측정 함수
# 외부에서 `source scripts/kpi-lib.sh` 로 포함해서 사용한다.
set -u

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# active_mvp.txt 포맷:
#   1줄 형태:  "11"               → 상태 미지정(기본값 in-progress로 간주)
#   2줄 형태:  "11\nin-progress"  → 명시적 상태
#   한줄 콜론: "11:in-progress"   → 한줄 형태
# 허용 상태: planning | in-progress | completing | done
#
# 상태별 Hard 게이트 적용 방침 (kpi_gate_resolve 참조):
#   planning    — 모든 Hard → Soft 강등. KPI 선언을 갓 작성하는 단계라 측정 불가가 정상.
#   in-progress — 점진 지표(커버리지·성능·회고)는 Soft로 강등, 테스트 통과·debt 등만 Hard 유지.
#   completing  — 원선언 그대로 (마일스톤 마감 직전이므로 전 지표 Hard 적용).
#   done        — 게이트 없음(다음 MVP로 이동해야 할 상태).

# active MVP 번호 반환 (없으면 빈 문자열)
kpi_active_mvp() {
  local f="$REPO_ROOT/progress/active_mvp.txt"
  [[ -f "$f" ]] || { echo ""; return; }
  # 첫 줄만, 콜론 뒤 상태는 제거
  head -1 "$f" | awk -F: '{gsub(/[[:space:]]/, "", $1); print $1}'
}

# active MVP 상태 반환 (기본: in-progress)
kpi_active_state() {
  local f="$REPO_ROOT/progress/active_mvp.txt"
  [[ -f "$f" ]] || { echo "in-progress"; return; }
  local first="$(head -1 "$f")"
  local second="$(sed -n '2p' "$f" | tr -d '[:space:]')"
  if [[ "$first" == *:* ]]; then
    echo "$first" | awk -F: '{gsub(/[[:space:]]/, "", $2); print $2}'
    return
  fi
  if [[ -n "$second" ]]; then
    echo "$second"
    return
  fi
  echo "in-progress"
}

# active 마일스톤 ID 반환 (예: "M2". 없으면 빈 문자열)
# 포맷: progress/active_milestone.txt에 "M2:in-progress" 또는 "M2\nin-progress"
kpi_active_milestone() {
  local f="$REPO_ROOT/progress/active_milestone.txt"
  [[ -f "$f" ]] || { echo ""; return; }
  head -1 "$f" | awk -F: '{gsub(/[[:space:]]/, "", $1); print $1}'
}

# active 마일스톤 상태 반환 (기본: in-progress)
kpi_active_milestone_state() {
  local f="$REPO_ROOT/progress/active_milestone.txt"
  [[ -f "$f" ]] || { echo "in-progress"; return; }
  local first="$(head -1 "$f")"
  local second="$(sed -n '2p' "$f" | tr -d '[:space:]')"
  if [[ "$first" == *:* ]]; then
    echo "$first" | awk -F: '{gsub(/[[:space:]]/, "", $2); print $2}'
    return
  fi
  if [[ -n "$second" ]]; then
    echo "$second"
    return
  fi
  echo "in-progress"
}

# 현재 마일스톤 기획 문서 경로 반환
# 우선순위: progress/mvp{N}/M{X}_*.md → progress/M{X}_*.md → progress/milestones/*/M{X}_*.md
kpi_find_milestone_doc() {
  local mvp="$1" milestone="$2"
  [[ -n "$mvp" && -n "$milestone" ]] || return 1
  local candidates=(
    "$REPO_ROOT/progress/mvp${mvp}/${milestone}_"*.md
    "$REPO_ROOT/progress/mvp${mvp}/${milestone}.md"
    "$REPO_ROOT/progress/${milestone}_"*.md
    "$REPO_ROOT/progress/milestones"/*/"${milestone}_"*.md
  )
  for f in "${candidates[@]}"; do
    [[ -f "$f" ]] && { echo "$f"; return 0; }
  done
  return 1
}

# MVP 최종 KPI 문서 경로 반환 (MVP 전체 목표 — completing/done 시 검증)
# 우선순위: progress/mvp{N}/_roadmap.md → progress/mvp{N}/_vision.md → kpi_find_plan_doc과 동일
kpi_find_mvp_final_doc() {
  local mvp="$1"
  [[ -n "$mvp" ]] || return 1
  local candidates=(
    "$REPO_ROOT/progress/mvp${mvp}/_roadmap.md"
    "$REPO_ROOT/progress/mvp${mvp}/_vision.md"
    "$REPO_ROOT/history/mvp${mvp}/_roadmap.md"
  )
  for f in "${candidates[@]}"; do
    [[ -f "$f" ]] && { echo "$f"; return 0; }
  done
  kpi_find_plan_doc "$mvp"
}

# MVP 기획 문서 경로 반환 (여러 후보 중 최초 매치)
kpi_find_plan_doc() {
  local mvp="$1"
  [[ -n "$mvp" ]] || return 1
  local candidates=(
    "$REPO_ROOT/progress"/*"MVP${mvp}"*.md
    "$REPO_ROOT/progress"/*"mvp${mvp}"*.md
    "$REPO_ROOT/progress/mvp${mvp}"/_roadmap.md
    "$REPO_ROOT/progress/mvp${mvp}"/_vision.md
    "$REPO_ROOT/history/mvp${mvp}"/_roadmap.md
  )
  for f in "${candidates[@]}"; do
    [[ -f "$f" ]] && { echo "$f"; return 0; }
  done
  return 1
}

# 기획 문서에서 ## KPI 섹션의 테이블 행만 추출 (stdin: 파일 내용)
# 출력: 각 행 `지표|측정|목표|게이트|기준선` (파이프 구분)
kpi_parse_table() {
  awk '
    BEGIN { in_kpi = 0; skip_header = 0 }
    /^##[[:space:]]+KPI/ { in_kpi = 1; skip_header = 0; next }
    in_kpi && /^##[[:space:]]/ { in_kpi = 0 }
    in_kpi && /^\|.*\|[[:space:]]*$/ {
      if (skip_header < 2) { skip_header++; next }
      gsub(/^[[:space:]]*\|[[:space:]]*/, "")
      gsub(/[[:space:]]*\|[[:space:]]*$/, "")
      gsub(/[[:space:]]*\|[[:space:]]*/, "|")
      print
    }
  '
}

# 상태·선언게이트·측정키·측정값 → 실제 적용할 게이트 결정
# 인자: $1=state, $2=선언 gate, $3=measure_key, $4=current_value
# 출력: "Hard" | "Soft"
# 규칙:
#   1) planning/done: 전 지표 Soft (선언 초안 작성 단계 또는 아카이브)
#   2) in-progress: 점진 지표(커버리지·회고·debt·manual)는 Soft로 강등
#   3) 측정 불가(current = - | missing | 빈값)이고 completing이 아니면 Soft 강등
#      (캐시 부재로 모든 커밋이 차단되는 함정 방지)
#   4) completing: 원선언 그대로 (마일스톤 마감 직전 엄격 적용)
kpi_gate_resolve() {
  local state="$1" declared="$2" key="$3" current="$4"
  if [[ "$state" == "planning" || "$state" == "done" ]]; then
    echo "Soft"; return
  fi
  if [[ "$state" == "in-progress" ]]; then
    case "$key" in
      server_cov|web_cov|ios_cov|retro_exists|debt_count|manual)
        echo "Soft"; return ;;
    esac
  fi
  if [[ "$state" != "completing" ]]; then
    if [[ -z "$current" || "$current" == "-" || "$current" == "missing" ]]; then
      echo "Soft"; return
    fi
  fi
  echo "$declared"
}

# 지표명 → 측정 방식 매핑 키 추론 (stdout: 키, 없으면 "manual")
kpi_measure_key() {
  local metric="$1"
  local lc
  lc="$(echo "$metric" | tr '[:upper:]' '[:lower:]')"
  case "$lc" in
    *"서버"*"커버리지"*|*"rust"*"coverage"*) echo "server_cov" ;;
    *"웹"*"커버리지"*|*"web"*"coverage"*)    echo "web_cov" ;;
    *"ios"*"커버리지"*|*"ios coverage"*)     echo "ios_cov" ;;
    *"회고"*"작성"*|*"retro"*)               echo "retro_exists" ;;
    *"기술부채"*|*"debt"*)                   echo "debt_count" ;;
    *"p50"*|*"로딩"*|*"성능"*|*"performance"*) echo "manual" ;;
    *"성공률"*|*"e2e"*|*"수집"*)              echo "manual" ;;
    *) echo "manual" ;;
  esac
}

# 실측: 서버 커버리지 (캐시 파일 우선, 없으면 빈값)
kpi_measure_server_cov() {
  local cache="$REPO_ROOT/server/target/tarpaulin/cobertura.xml"
  if [[ -f "$cache" ]]; then
    awk -F'line-rate="' 'NR==1 || /<coverage/ {split($2,a,"\""); if (a[1] != "") { printf "%.1f%%\n", a[1]*100; exit } }' "$cache" 2>/dev/null
  fi
}

# 실측: 웹 커버리지 (coverage-summary.json 사용)
kpi_measure_web_cov() {
  local cache="$REPO_ROOT/web/coverage/coverage-summary.json"
  if [[ -f "$cache" ]]; then
    # "lines":{"total":X,"covered":Y,"pct":Z}
    awk 'match($0,/"lines":[^}]*"pct":[[:space:]]*[0-9.]+/) {
      s=substr($0, RSTART, RLENGTH); sub(/.*"pct":[[:space:]]*/, "", s); printf "%s%%\n", s; exit
    }' "$cache" 2>/dev/null
  fi
}

# 실측: iOS 커버리지 (coverage.sh 결과 파일 기준)
kpi_measure_ios_cov() {
  local cache="$REPO_ROOT/ios/Frank/coverage.txt"
  [[ -f "$cache" ]] && head -1 "$cache" 2>/dev/null
}

# 회고 존재: history/mvp{N}/*retro* 파일 유무
kpi_measure_retro_exists() {
  local mvp="$1"
  local dir="$REPO_ROOT/history/mvp${mvp}"
  [[ -d "$dir" ]] || { echo "missing"; return; }
  if ls "$dir"/*retro* >/dev/null 2>&1; then echo "exists"; else echo "missing"; fi
}

# 기술부채 카운트
kpi_measure_debt_count() {
  local f="$REPO_ROOT/progress/debt.md"
  [[ -f "$f" ]] || { echo "-"; return; }
  grep -cE '^(- \[ \]|- \[x\])' "$f" 2>/dev/null || echo "0"
}

# 목표 비교: target에 ≥N%, ≤Ns, N+ 형식 지원. 달성이면 0, 미달이면 1 반환
# 사용: kpi_eval_target "$target" "$current"
kpi_eval_target() {
  local target="$1" current="$2"
  # target에 공백만 제거 (%/s 등 단위는 아래에서 패턴별 처리)
  local t="${target//[[:space:]]/}"
  local c="${current//[[:space:]]/}"

  # 비숫자 특수 케이스는 먼저 처리
  case "$t" in
    "exists"|"exist") [[ "$c" == "exists" ]] && return 0 || return 1 ;;
    "net감소"|"net减少"|"감소") return 2 ;; # 수동 판단
  esac

  [[ -z "$c" || "$c" == "-" || "$c" == "missing" ]] && return 2 # 측정 불가

  # 숫자 비교용 단위 제거
  t="${t//%/}"; c="${c//%/}"
  t="${t%s}";  c="${c%s}"

  if [[ "$t" =~ ^≥([0-9.]+) ]]; then
    awk -v c="$c" -v t="${BASH_REMATCH[1]}" 'BEGIN { exit !(c+0 >= t+0) }' && return 0 || return 1
  elif [[ "$t" =~ ^≤([0-9.]+) ]]; then
    awk -v c="$c" -v t="${BASH_REMATCH[1]}" 'BEGIN { exit !(c+0 <= t+0) }' && return 0 || return 1
  elif [[ "$t" =~ ^([0-9]+)\+?$ ]]; then
    awk -v c="$c" -v t="${BASH_REMATCH[1]}" 'BEGIN { exit !(c+0 >= t+0) }' && return 0 || return 1
  fi
  return 2
}
