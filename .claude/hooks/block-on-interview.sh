#!/bin/bash
# PreToolUse hook (matcher: Edit|Write) — 인터뷰 미완료 시 Edit/Write 차단
#
# 목적: step-1/step-4 인터뷰 진행 도중 사용자와 한 질문 깊이 토론하다
#       남은 질문 잊고 구현으로 직진하는 망각 패턴을 물리적으로 차단.
#
# Phase 3 — 차단형. exit 2로 tool 호출을 막고 stderr로 안내 메시지 전달.
#
# 안전장치 6개:
#   S1 matcher 좁히기 (Edit|Write만)         — settings.json matcher가 처리
#   S2 활성 step 체크                        — active_step.txt 비었으면 통과
#   S3 step 전환 시 자동 cleanup             — 각 SKILL이 라이프사이클 책임 (외부)
#   S4 메타태스크 면제                       — .claude/, scripts/, rules/, progress/meta/, history/, 메모류
#   S5 bypass 환경변수 (WORKFLOW_BYPASS=1)
#   S6 자동 expire (24h 넘으면 stale, 통과)
#
# 입력: stdin으로 PreToolUse event JSON ({tool_name, tool_input: {file_path, ...}, ...})
# 출력: 정상 → exit 0 (무음). 차단 → stderr 메시지 + exit 2.

set -e
ROOT="${CLAUDE_PROJECT_DIR:-$(pwd)}"

# S5: bypass 환경변수 — 모든 검사 건너뛰고 통과
[ "$WORKFLOW_BYPASS" = "1" ] && exit 0

# S2: 활성 step 없으면 통과
STEP_FILE="$ROOT/progress/active_step.txt"
[ -f "$STEP_FILE" ] || exit 0
STEP=$(tr -d '[:space:]' < "$STEP_FILE")
[ -z "$STEP" ] || [ "$STEP" = "none" ] && exit 0

# 인터뷰 상태 확인
INT_FILE="$ROOT/progress/active_interview.json"
[ -f "$INT_FILE" ] || exit 0

CUR=$(jq -r '.current // 0' "$INT_FILE" 2>/dev/null || echo 0)
TOT=$(jq -r '.total // 0' "$INT_FILE" 2>/dev/null || echo 0)

# 인터뷰 없거나 완료된 상태면 통과
[ "$TOT" = "0" ] && exit 0
[ "$CUR" -ge "$TOT" ] && exit 0

# S6: 24h expire — started_at + 86400 < 현재면 stale, 통과
STARTED=$(jq -r '.started_at // empty' "$INT_FILE" 2>/dev/null || echo "")
if [ -n "$STARTED" ]; then
  NOW=$(date -u +%s)
  # macOS BSD date: -j -f 형식 사용 (GNU date -d 와 다름)
  START_EPOCH=$(date -u -j -f "%Y-%m-%dT%H:%M:%SZ" "$STARTED" +%s 2>/dev/null || echo 0)
  if [ "$START_EPOCH" -gt 0 ]; then
    AGE=$(( NOW - START_EPOCH ))
    if [ "$AGE" -gt 86400 ]; then
      exit 0
    fi
  fi
fi

# S4: 메타태스크 면제 (tool_input의 file_path 검사)
INPUT=$(cat)
TOOL_INPUT=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null || echo "")

case "$TOOL_INPUT" in
  *"/.claude/"*|*"/scripts/"*|*"/rules/"*|*"/progress/meta/"*|*"/history/"*)
    exit 0 ;;
  */progress/notes.md|*/progress/bugs.md|*/progress/debts.md|*/progress/future-ideas.md)
    exit 0 ;;
esac

# 차단 — stderr로 안내 메시지 전달
REM=$(jq -r '.remaining // [] | join(", ")' "$INT_FILE" 2>/dev/null || echo "")
{
  echo "❌ 인터뷰 Q$CUR/$TOT 미완료 — Edit/Write 차단"
  [ -n "$REM" ] && echo "   남은 질문: $REM"
  echo ""
  echo "   해결책 1: 인터뷰 마저 진행 (사용자에게 남은 질문 제시)"
  echo "   해결책 2 (긴급): WORKFLOW_BYPASS=1 환경변수로 우회"
  echo ""
  echo "   대상 파일: $TOOL_INPUT"
} >&2
exit 2
