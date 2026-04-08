#!/usr/bin/env bash
# coverage.sh — Frank iOS 커버리지 측정 파이프라인
#
# 사용법:
#   cd <repo-root>
#   bash scripts/coverage.sh
#
# FrankTests(단위) + FrankUITests(E2E) 통합 실행 후
# Frank 앱 타겟의 lineCoverage를 집계한다.
# 90% 미만이면 exit 1로 실패한다.

set -euo pipefail

WORKSPACE="ios/Frank/Frank.xcworkspace"
SCHEME="Frank"
DESTINATION="platform=iOS Simulator,name=iPhone 17 Pro"
RESULT_BUNDLE="/tmp/frank_coverage.xcresult"
APP_TARGET="Frank"
THRESHOLD=90

echo "=== Frank iOS 커버리지 측정 ==="
echo "  워크스페이스: $WORKSPACE"
echo "  타겟        : $APP_TARGET"
echo "  목표        : ${THRESHOLD}%"
echo ""

# 1. 기존 결과 제거
rm -rf "$RESULT_BUNDLE"

# 2. FrankTests + FrankUITests 통합 실행
echo "[1/2] 테스트 실행 (FrankTests + FrankUITests)..."
xcodebuild test \
  -workspace "$WORKSPACE" \
  -scheme "$SCHEME" \
  -destination "$DESTINATION" \
  -enableCodeCoverage YES \
  -resultBundlePath "$RESULT_BUNDLE" \
  -quiet 2>&1 | grep -E "(error:|warning:|Test Suite|PASS|FAIL|Executed)" || true

echo ""

# 3. 커버리지 집계
echo "[2/2] 커버리지 집계..."
COVERAGE=$(xcrun xccov view --report --json "$RESULT_BUNDLE" \
  | python3 - <<'PYEOF'
import json, sys
data = json.load(sys.stdin)
for target in data.get("targets", []):
    if target.get("name") == "Frank":
        cov = target.get("lineCoverage", 0)
        print(f"{cov * 100:.1f}")
        sys.exit(0)
# 매칭 타겟 없으면 0
print("0.0")
PYEOF
)

COVERAGE_INT=$(echo "$COVERAGE" | python3 -c "import sys; print(int(float(sys.stdin.read())))")

echo ""
echo "=== 커버리지 결과 ==="
echo "  타겟   : $APP_TARGET"
echo "  수치   : ${COVERAGE}%"
echo "  목표   : ${THRESHOLD}%"
echo ""

if [ "$COVERAGE_INT" -lt "$THRESHOLD" ]; then
  echo "❌ 커버리지 ${COVERAGE}% < 목표 ${THRESHOLD}% — 테스트를 추가하세요."
  exit 1
else
  echo "✅ 커버리지 ${COVERAGE}% >= 목표 ${THRESHOLD}%"
fi
