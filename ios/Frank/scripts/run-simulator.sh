#!/bin/bash
set -euo pipefail

SIMULATOR_NAME="${1:-iPhone 17 Pro}"
SCHEME="Frank"
BUNDLE_ID="dev.frank.app"
WORKSPACE="$(cd "$(dirname "$0")/.." && pwd)/Frank.xcworkspace"
DERIVED_DATA="/tmp/FrankBuild"

echo "▶ Booting $SIMULATOR_NAME..."
xcrun simctl boot "$SIMULATOR_NAME" 2>/dev/null || true
open -a Simulator

echo "▶ Building $SCHEME..."
xcodebuild build \
  -workspace "$WORKSPACE" \
  -scheme "$SCHEME" \
  -destination "platform=iOS Simulator,name=$SIMULATOR_NAME" \
  -derivedDataPath "$DERIVED_DATA" \
  -quiet

echo "▶ Installing & launching..."
xcrun simctl install "$SIMULATOR_NAME" "$DERIVED_DATA/Build/Products/Debug-iphonesimulator/Frank.app"
xcrun simctl launch "$SIMULATOR_NAME" "$BUNDLE_ID"

echo "✔ Frank running on $SIMULATOR_NAME"
