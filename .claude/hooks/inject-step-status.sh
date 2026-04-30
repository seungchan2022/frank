#!/bin/bash
# UserPromptSubmit hook — 활성 step + 인터뷰 진행 상태를 시스템 메시지로 자동 주입
#
# 목적: LLM이 자기 컨텍스트에서 인터뷰 망각하는 문제 완화.
#       매 사용자 프롬프트마다 첫 줄에 현재 상태를 prepend.
#
# Phase 2 — 차단 없음, 안내만 (Phase 3 PreToolUse hook이 차단 담당).
#
# 입력: stdin으로 hook event JSON (CLAUDE_PROJECT_DIR 환경변수 기반 경로)
# 출력: stdout으로 {"systemMessage": "..."} JSON (활성 step 없으면 출력 없이 exit 0)

set -e
ROOT="${CLAUDE_PROJECT_DIR:-$(pwd)}"

STEP_FILE="$ROOT/progress/active_step.txt"
INT_FILE="$ROOT/progress/active_interview.json"

# 활성 step 없으면 무음 통과
[ -f "$STEP_FILE" ] || exit 0
STEP=$(tr -d '[:space:]' < "$STEP_FILE")
[ -z "$STEP" ] || [ "$STEP" = "none" ] && exit 0

# 인터뷰 진행 중인지 확인
if [ -f "$INT_FILE" ]; then
  CUR=$(jq -r '.current // empty' "$INT_FILE" 2>/dev/null || echo "")
  TOT=$(jq -r '.total // empty' "$INT_FILE" 2>/dev/null || echo "")
  if [ -n "$CUR" ] && [ -n "$TOT" ]; then
    REM=$(jq -r '.remaining // [] | join(", ")' "$INT_FILE" 2>/dev/null || echo "")
    if [ -n "$REM" ]; then
      MSG="📝 활성: $STEP | 인터뷰 Q$CUR/$TOT | 남은 질문: $REM"
    else
      MSG="📝 활성: $STEP | 인터뷰 Q$CUR/$TOT"
    fi
    jq -n --arg msg "$MSG" '{systemMessage: $msg}'
    exit 0
  fi
fi

# 인터뷰 없는 step만 진행 중
jq -n --arg step "$STEP" '{systemMessage: ("📝 활성 step: " + $step)}'
exit 0
