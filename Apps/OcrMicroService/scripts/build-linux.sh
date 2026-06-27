#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="ocr-microservice-builder"
CONTAINER_NAME="ocr-microservice-build-output"
BINARY_NAME="OcrMicroService"

docker build -f Dockerfile.build -t "$IMAGE_NAME" .

docker rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || true
docker create --name "$CONTAINER_NAME" "$IMAGE_NAME" >/dev/null

mkdir -p dist
docker cp "$CONTAINER_NAME:/app/target/release/$BINARY_NAME" "dist/$BINARY_NAME"

docker rm "$CONTAINER_NAME" >/dev/null

echo "Built dist/$BINARY_NAME"
