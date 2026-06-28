#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 user@host /remote/install/path"
  exit 1
fi

TARGET_HOST="$1"
TARGET_DIR="$2"
BINARY_NAME="OcrMicroService"

ssh "$TARGET_HOST" "mkdir -p '$TARGET_DIR'"

scp "dist/$BINARY_NAME" "$TARGET_HOST:$TARGET_DIR/$BINARY_NAME"
scp "config_examples/tesseract_docker.toml" "$TARGET_HOST:$TARGET_DIR/tesseract.toml"

ssh "$TARGET_HOST" "chmod +x '$TARGET_DIR/$BINARY_NAME'"

echo "Deployed to $TARGET_HOST:$TARGET_DIR"

