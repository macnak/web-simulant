#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$ROOT_DIR/target/release/web-simulant"

if [[ ! -x "$BIN" ]]; then
  echo "Release binary not found. Building..."
  (cd "$ROOT_DIR" && cargo build --release)
fi

"$BIN" &
SERVER_PID=$!

cleanup() {
  if kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID"
  fi
}
trap cleanup EXIT

for _ in {1..20}; do
  if curl -sSf http://localhost:8081/api/health >/dev/null; then
    break
  fi
  sleep 0.25
done

curl -sS http://localhost:8081/api/health

echo ""

echo "Loading: examples/01-simple-health-check.yaml"
curl -sS -F "config=@$ROOT_DIR/examples/01-simple-health-check.yaml" \
  http://localhost:8081/api/config/import/multipart
curl -sS http://localhost:8080/health

echo ""

echo "Loading: examples/02-user-api-basic.yaml"
curl -sS -F "config=@$ROOT_DIR/examples/02-user-api-basic.yaml" \
  http://localhost:8081/api/config/import/multipart
curl -sS http://localhost:8080/api/users

echo ""

echo "Loading: examples/03-ecommerce-mixed.yaml"
curl -sS -F "config=@$ROOT_DIR/examples/03-ecommerce-mixed.yaml" \
  http://localhost:8081/api/config/import/multipart
curl -sS http://localhost:8080/api/products

echo ""

echo "Loading: examples/04-unreliable-external.yaml"
curl -sS -F "config=@$ROOT_DIR/examples/04-unreliable-external.yaml" \
  http://localhost:8081/api/config/import/multipart
curl -sS http://localhost:8080/api/weather

echo ""

echo "Smoke test complete."
