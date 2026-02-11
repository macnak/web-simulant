#!/bin/bash
# High concurrency stress test

set -e

SERVER_URL="${SERVER_URL:-http://localhost:8080}"
ENDPOINT="${ENDPOINT:-/simulate}"
URL="${SERVER_URL}${ENDPOINT}"

echo "=== High Concurrency Stress Test ==="
echo "Testing at 5k and 10k concurrent connections"
echo ""

# 5k concurrent
echo "Test 1: 5k concurrent, 50ms latency (50k total requests)"
oha -n 50000 -c 5000 \
    -m POST \
    -H "Content-Type: application/json" \
    -d '{"latency_ms": 50, "error_rate": 0.0}' \
    "$URL"

echo ""
echo "Test 2: 10k concurrent, 50ms latency (100k total requests)"
echo "Warning: This may saturate resources - monitor CPU and memory"
oha -n 100000 -c 10000 \
    -m POST \
    -H "Content-Type: application/json" \
    -d '{"latency_ms": 50, "error_rate": 0.0}' \
    "$URL"

echo ""
echo "=== High Concurrency Complete ==="
echo "Check for: CPU usage, memory stability, distribution accuracy"
