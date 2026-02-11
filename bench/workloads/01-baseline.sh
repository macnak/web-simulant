#!/bin/bash
# Baseline workload: measure natural overhead with no artificial delay

set -e

SERVER_URL="${SERVER_URL:-http://localhost:8080}"
ENDPOINT="${ENDPOINT:-/simulate}"
URL="${SERVER_URL}${ENDPOINT}"

echo "=== Baseline Workload ==="
echo "Measuring natural overhead (0ms configured latency)"
echo ""

# Low concurrency baseline
echo "Test 1: Low concurrency (10 concurrent, 100 requests)"
oha -n 100 -c 10 \
    -m POST \
    -H "Content-Type: application/json" \
    -d '{"latency_ms": 0, "error_rate": 0.0}' \
    "$URL"

echo ""
echo "Test 2: Moderate concurrency (100 concurrent, 1000 requests)"
oha -n 1000 -c 100 \
    -m POST \
    -H "Content-Type: application/json" \
    -d '{"latency_ms": 0, "error_rate": 0.0}' \
    "$URL"

echo ""
echo "Test 3: High concurrency (1000 concurrent, 10000 requests)"
oha -n 10000 -c 1000 \
    -m POST \
    -H "Content-Type: application/json" \
    -d '{"latency_ms": 0, "error_rate": 0.0}' \
    "$URL"

echo ""
echo "=== Baseline Complete ==="
