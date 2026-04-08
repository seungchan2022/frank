#!/bin/bash
# Apple Client Secret 만료일 확인 스크립트 (macOS 전용)
#
# 사용법:
#   ./scripts/check_apple_secret.sh
#   APPLE_CLIENT_SECRET_EXPIRES_AT=2026-10-08 ./scripts/check_apple_secret.sh

set -euo pipefail

EXPIRES_AT="${APPLE_CLIENT_SECRET_EXPIRES_AT:-}"

if [ -z "$EXPIRES_AT" ]; then
  echo "❌ Error: APPLE_CLIENT_SECRET_EXPIRES_AT not set"
  echo "   환경변수로 직접 전달하거나 .env에서 export 후 실행하세요."
  exit 1
fi

# macOS date -j 로 YYYY-MM-DD 파싱 → Unix timestamp
EXPIRY_EPOCH=$(date -j -f "%Y-%m-%d" "$EXPIRES_AT" "+%s" 2>/dev/null || echo "")

if [ -z "$EXPIRY_EPOCH" ]; then
  echo "❌ Error: 날짜 형식이 올바르지 않습니다: '$EXPIRES_AT'"
  echo "   올바른 형식: YYYY-MM-DD (예: 2026-10-08)"
  exit 1
fi

NOW_EPOCH=$(date +%s)
DAYS_LEFT=$(( (EXPIRY_EPOCH - NOW_EPOCH) / 86400 ))

echo ""
echo "Apple Client Secret Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Expires at : $EXPIRES_AT"
echo "Days left  : $DAYS_LEFT"
echo ""

if [ "$DAYS_LEFT" -lt 0 ]; then
  echo "🔴 EXPIRED — Apple login is broken. 즉시 갱신 필요."
  echo "   Run: node scripts/generate_apple_secret.js ..."
  exit 2
elif [ "$DAYS_LEFT" -le 7 ]; then
  echo "🔴 CRITICAL: $DAYS_LEFT 일 후 만료 — 즉시 갱신 필요."
  echo "   Run: node scripts/generate_apple_secret.js ..."
  exit 1
elif [ "$DAYS_LEFT" -le 30 ]; then
  echo "🟠 WARNING: $DAYS_LEFT 일 후 만료 — 갱신 계획을 세우세요."
  echo "   Run: node scripts/generate_apple_secret.js ..."
elif [ "$DAYS_LEFT" -le 60 ]; then
  echo "🟡 NOTICE: $DAYS_LEFT 일 후 만료 — 갱신 준비를 시작하세요."
else
  echo "🟢 OK: $DAYS_LEFT 일 후 만료 (이상 없음)."
fi
