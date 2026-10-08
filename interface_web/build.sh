#!/usr/bin/env bash
set -e

APP_NAME="H2GP"
TARGET_DIR="target/release/bundle/osx"
APP_PATH="$TARGET_DIR/$APP_NAME.app"
DMG_PATH="$APP_NAME.dmg"

echo "== Bundling app =="
cargo bundle --release --bin interface

echo "== Ad-hoc signing app =="
codesign --force --deep --sign - "$APP_PATH"

echo "== Verifying signature =="
codesign --verify --deep --strict --verbose=4 "$APP_PATH"

echo "== Creating DMG =="
hdiutil create "$DMG_PATH" \
  -volname "$APP_NAME" \
  -srcfolder "$APP_PATH" \
  -ov -format UDZO

echo "== Done =="
echo "DMG created at: $DMG_PATH"
