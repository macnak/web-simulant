#!/bin/bash
# Error injection workload: test 1%, 5%, 10% error rates

set -e

SERVER_URL="${SERVER_URL:-http://localhost:8080}"
ENDPOINT="${ENDPOINT:-/simulate}"
URL="${SERVER_URL}${ENDPOINT}"

echo "=== Error Injection Workload ==="
echo "Testing error rates with 50ms latency"
echo ""

# 1% error rate
echo "Test 1: 1% error rate (100 concurrent, 1000 requests)"
oha -n 1000 -c 100 \
    -m POST \
    -H "Content-Type: application/json" \
    -d '{"latency_ms": 50, "error_rate": 0.01}' \
    "$URL"

echo ""
echo "Test 2: 5% error rate (100 concurrent, 1000 requests)"
oha -n 1000 -c 100 \
    -m POST \
    -H "Content-Type: application/json" \
    -d '{"latency_ms": 50, "error_rate": 0.05}' \
    "$URL"

echo ""
echo "Test 3: 10% error rate (100 concurrent, 1000 requests)"
oha -n 1000 -c 100 \
    -m POST \
    -H "Content-Type: application/json" \
    -d '{"latency_ms": 50, "error_rate": 0.10}' \
    "$URL"

echo ""
echo "=== Error Injection Complete ==="
echo "Expected: error rates should match configured rates within tolerance"
