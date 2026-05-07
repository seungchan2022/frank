#!/bin/bash
# PostToolUse hook — Skill 호출 시 active_step.txt 자동 갱신 (DEBT-HOOK-02)
#
# 목적: step-N 스킬 호출 시 Claude 의존 없이 훅 레벨에서 자동 갱신.
#       DEBT-HOOK-01 A안(SKILL.md 지시)의 백업 역할.
#
# 필터: tool_name == "Skill" && tool_input.skill =~ ^step-[1-9]$ 만 갱신.
#       /next, /workflow, /init, /status 등 다른 Skill 호출은 갱신 안 함.
#
# 입력: stdin으로 PostToolUse event JSON
# 출력: 없음 (silent)

INPUT=$(cat)
TOOL=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null)
[ "$TOOL" = "Skill" ] || exit 0

SKILL=$(echo "$INPUT" | jq -r '.tool_input.skill // empty' 2>/dev/null)
echo "$SKILL" | grep -qE '^step-[1-9]$' || exit 0

ROOT="${CLAUDE_PROJECT_DIR:-$(pwd)}"
echo "$SKILL" > "$ROOT/progress/active_step.txt"
