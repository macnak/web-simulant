#!/bin/bash
# Fixed latency workload: test 10ms, 50ms, 100ms delays

set -e

SERVER_URL="${SERVER_URL:-http://localhost:8080}"
ENDPOINT="${ENDPOINT:-/simulate}"
URL="${SERVER_URL}${ENDPOINT}"

echo "=== Fixed Latency Workload ==="
echo "Testing fixed delays at moderate concurrency"
echo ""

# 10ms fixed latency
echo "Test 1: 10ms fixed latency (100 concurrent, 1000 requests)"
oha -n 1000 -c 100 \
    -m POST \
    -H "Content-Type: application/json" \
    -d '{"latency_ms": 10, "error_rate": 0.0}' \
    "$URL"

echo ""
echo "Test 2: 50ms fixed latency (100 concurrent, 1000 requests)"
oha -n 1000 -c 100 \
    -m POST \
    -H "Content-Type: application/json" \
    -d '{"latency_ms": 50, "error_rate": 0.0}' \
    "$URL"

echo ""
echo "Test 3: 100ms fixed latency (100 concurrent, 1000 requests)"
oha -n 1000 -c 100 \
    -m POST \
    -H "Content-Type: application/json" \
    -d '{"latency_ms": 100, "error_rate": 0.0}' \
    "$URL"

echo ""
echo "=== Fixed Latency Complete ==="
echo "Expected: p50/p95/p99 should be close to configured latency"
echo "Tolerance: within 10% for >=100 samples"
