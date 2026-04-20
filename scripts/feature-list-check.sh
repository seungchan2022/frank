#!/usr/bin/env bash
# feature-list-check.sh — Feature List 미체크 시 커밋 차단
#
# 동작:
#   1) progress/active_subtask.txt 읽어 대상 서브태스크 문서 경로 획득
#   2) 해당 문서의 ## Feature List 섹션 파싱
#   3) 미체크 [ ] 항목 1개 이상이면 에러 출력 + exit 1
#   4) 파싱 실패(포맷 위반) 시 차단 + 위치 안내
#
# 우회:
#   FEATURE_LIST_BYPASS=1 git commit ...
#   → progress/feature-list/bypass.log 에 사유 기록 필수
#
# 참고:
#   - pre-commit hook에서 "문서만 변경" 판정(NON_DOC_FILES 없음)이 이미 된 경우에만 도달
#   - 따라서 docs:/chore: 커밋은 pre-commit에서 이미 통과처리됨 (이 스크립트 미호출)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

ACTIVE_SUBTASK_FILE="$REPO_ROOT/progress/active_subtask.txt"
BYPASS_LOG_DIR="$REPO_ROOT/progress/feature-list"
BYPASS_LOG="$BYPASS_LOG_DIR/bypass.log"

# ── 우회 처리 ──────────────────────────────────────────────────────────────
BYPASS="${FEATURE_LIST_BYPASS:-0}"
if [[ "$BYPASS" == "1" ]]; then
  mkdir -p "$BYPASS_LOG_DIR"
  SUBTASK_REL_BYPASS="$(cat "$ACTIVE_SUBTASK_FILE" 2>/dev/null | tr -d '[:space:]' || echo 'unknown')"
  echo "$(date +%Y-%m-%dT%H:%M:%S) BYPASS | channel: hook | subtask: ${SUBTASK_REL_BYPASS} | unchecked: ? | reason: FEATURE_LIST_BYPASS=1 (사유를 이 파일에 직접 추가하세요)" >>"$BYPASS_LOG"
  echo "[feature-list-check] FEATURE_LIST_BYPASS=1 — 검증 스킵"
  echo "  사유를 $BYPASS_LOG 에 추가 기록해 주세요."
  exit 0
fi

# ── active_subtask.txt 확인 ────────────────────────────────────────────────
if [[ ! -f "$ACTIVE_SUBTASK_FILE" ]]; then
  # 파일 없으면 통과 (아직 §8.2 이전 서브태스크 또는 파일 미생성)
  exit 0
fi

SUBTASK_REL="$(cat "$ACTIVE_SUBTASK_FILE" | tr -d '[:space:]')"

if [[ -z "$SUBTASK_REL" || "$SUBTASK_REL" == "none" ]]; then
  exit 0
fi

# ── 경로 검증 — repo 밖 파일 접근 방지 ───────────────────────────────────
if [[ "$SUBTASK_REL" != progress/subtask/* ]] || [[ "$SUBTASK_REL" == *".."* ]]; then
  echo "❌ active_subtask.txt 경로 위반: progress/subtask/ 접두사 필수, '..' 금지"
  echo "   현재 값: $SUBTASK_REL"
  echo "→ active_subtask.txt 를 올바른 progress/subtask/ 경로로 수정한 뒤 재진행하세요."
  exit 1
fi

SUBTASK_DOC="$REPO_ROOT/$SUBTASK_REL"

if [[ ! -f "$SUBTASK_DOC" ]]; then
  echo "[feature-list-check] 경고: active_subtask.txt 경로의 파일을 찾을 수 없음: $SUBTASK_REL"
  echo "  → 검증 생략 (통과)"
  exit 0
fi

# ── Feature List 섹션 추출 ─────────────────────────────────────────────────
# "## Feature List" 이후 다음 "## " 섹션 전까지 추출
FEATURE_SECTION=""
in_section=0
while IFS= read -r line; do
  if [[ "$line" =~ ^##[[:space:]]Feature[[:space:]]List ]]; then
    in_section=1
    continue
  fi
  if [[ $in_section -eq 1 ]]; then
    if [[ "$line" =~ ^##[[:space:]] ]]; then
      break
    fi
    FEATURE_SECTION+="$line"$'\n'
  fi
done <"$SUBTASK_DOC"

if [[ -z "$FEATURE_SECTION" ]]; then
  # Feature List 섹션 없으면 통과
  exit 0
fi

# ── HTML 메타 파싱 ─────────────────────────────────────────────────────────
META_LINE="$(echo "$FEATURE_SECTION" | grep -E '^<!--.*size.*count.*skip.*-->' | head -1 || true)"

if [[ -z "$META_LINE" ]]; then
  echo "❌ Feature List 파싱 실패 — HTML 메타 누락"
  echo "   문서: $SUBTASK_REL"
  echo "   필요 포맷: <!-- size: {소형|중형|대형} | count: {N} | skip: {true|false} -->"
  echo ""
  echo "→ 서브태스크 문서에 HTML 메타를 추가한 뒤 재진행하세요."
  exit 1
fi

SKIP_VAL="$(echo "$META_LINE" | sed -n 's/.*skip:[[:space:]]*\([a-z]*\).*/\1/p')"
COUNT_META="$(echo "$META_LINE" | sed -n 's/.*count:[[:space:]]*\([0-9]*\).*/\1/p')"

if [[ "$SKIP_VAL" == "true" ]]; then
  exit 0
fi

# ── 체크박스 라인 수집 ─────────────────────────────────────────────────────
# 허용 상태 기호: [ ] [x] [~] [-]
VALID_ID_PATTERN='[FERTUGP]-[0-9]{2}'

parse_errors=()
unchecked_lines=()
total_count=0

while IFS= read -r line; do
  # 체크박스 라인 감지 (grep으로 패턴 매칭 — bash =~ 이스케이프 문제 회피)
  if echo "$line" | grep -qE '^[[:space:]]*- \[ \]'; then
    state="unchecked"
  elif echo "$line" | grep -qE '^[[:space:]]*- \[x\]'; then
    state="checked"
  elif echo "$line" | grep -qE '^[[:space:]]*- \[~\]'; then
    state="deferred"
  elif echo "$line" | grep -qE '^[[:space:]]*- \[-\]'; then
    state="na"
  elif echo "$line" | grep -qE '^[[:space:]]*- \['; then
    # 허용 외 기호
    parse_errors+=("상태 기호 위반: $line")
    continue
  else
    continue
  fi

  total_count=$((total_count + 1))

  # ID 포맷 검증 (체크박스 내 ID 포함 여부)
  if ! echo "$line" | grep -qE "$VALID_ID_PATTERN"; then
    parse_errors+=("ID 포맷 위반 (${VALID_ID_PATTERN} 아님): $line")
    continue
  fi

  # [~] [- ] 사유 누락 검증
  # 올바른 포맷: [~] deferred (사유) {ID} ... / [-] N/A (사유) {ID} ...
  if [[ "$state" == "deferred" ]]; then
    if ! echo "$line" | grep -qE '\[~\][[:space:]]+deferred[[:space:]]*\(.+\)'; then
      parse_errors+=("[~] deferred (사유) 포맷 위반: $line")
    fi
  elif [[ "$state" == "na" ]]; then
    if ! echo "$line" | grep -qE '\[-\][[:space:]]+N/A[[:space:]]*\(.+\)'; then
      parse_errors+=("[-] N/A (사유) 포맷 위반: $line")
    fi
  fi

  if [[ "$state" == "unchecked" ]]; then
    unchecked_lines+=("$line")
  fi
done <<<"$FEATURE_SECTION"

# ── count 불일치 검증 ─────────────────────────────────────────────────────
if [[ -n "$COUNT_META" && "$total_count" -ne "$COUNT_META" ]]; then
  echo "❌ Feature List 파싱 실패 — count 불일치"
  echo "   문서: $SUBTASK_REL"
  echo "   HTML 메타 count: $COUNT_META  /  실제 체크박스: $total_count"
  echo ""
  echo "→ HTML 메타의 count 값을 실제 체크박스 수로 수정하거나"
  echo "   누락된 항목을 추가한 뒤 재진행하세요."
  exit 1
fi

# ── 파싱 에러 출력 ─────────────────────────────────────────────────────────
if [[ ${#parse_errors[@]} -gt 0 ]]; then
  echo "❌ Feature List 파싱 실패 — 포맷 위반 ${#parse_errors[@]}건"
  echo "   문서: $SUBTASK_REL"
  for err in "${parse_errors[@]}"; do
    echo "   · $err"
  done
  echo ""
  echo "→ 위반 항목을 수정한 뒤 재진행하세요."
  echo "   ID 규약: F/E/R/T/U/G/P-NN (예: F-01)"
  echo "   상태 기호: [ ] [x] [~] [-]"
  echo "   [~]/[-] 사유: [~] deferred (사유) F-01 ... 형식"
  exit 1
fi

# ── 미체크 항목 차단 ─────────────────────────────────────────────────────
if [[ ${#unchecked_lines[@]} -gt 0 ]]; then
  echo "❌ Feature List 미체크 항목 ${#unchecked_lines[@]}개 — 커밋 차단"
  echo "   문서: $SUBTASK_REL"
  echo ""
  for uline in "${unchecked_lines[@]}"; do
    echo "   $uline"
  done
  echo ""
  echo "→ 선택지:"
  echo "   1) /step-8 로 돌아가 항목 검증 후 재진입"
  echo "   2) 해당 항목을 [~] deferred (사유) 또는 [-] N/A (사유) 처리 후 재진입"
  echo "   3) FEATURE_LIST_BYPASS=1 git commit ... (사유를 progress/feature-list/bypass.log 에 기록)"
  exit 1
fi

echo "[feature-list-check] PASS — Feature List 미체크 없음"
exit 0
