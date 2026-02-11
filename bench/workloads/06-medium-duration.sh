#!/bin/bash
# Medium-duration stability test: 10-60 minutes at moderate TPS
# Purpose: Check stability and CPU usage without extreme load

set -e

SERVER_URL="${SERVER_URL:-http://localhost:8080}"
ENDPOINT="${ENDPOINT:-/simulate}"
URL="${SERVER_URL}${ENDPOINT}"
DURATION_MINUTES="${DURATION_MINUTES:-10}"
TPS="${TPS:-100}"
CONCURRENCY="${CONCURRENCY:-100}"

echo "=== Medium-Duration Stability Test ==="
echo "Duration: ${DURATION_MINUTES} minutes"
echo "Target TPS: ${TPS}"
echo "Concurrency: ${CONCURRENCY}"
echo "Latency: 50ms"
echo "Error rate: 1%"
echo ""
echo "Note: This test is indicative only. Results may differ significantly"
echo "      in Docker or on different hardware. Current environment: WSL2"
echo ""
echo "Starting at: $(date)"
echo ""

# Calculate total requests for duration
TOTAL_REQUESTS=$((TPS * 60 * DURATION_MINUTES))

echo "Total requests to send: ${TOTAL_REQUESTS}"
echo "Estimated completion: $(date -d "+${DURATION_MINUTES} minutes")"
echo ""
echo "Monitor CPU usage in another terminal:"
echo "  watch -n 2 'ps aux | grep web-simulant-bench | grep -v grep'"
echo ""

# Run the test with rate limiting
# Note: oha may not perfectly enforce TPS, this is approximate
oha -n ${TOTAL_REQUESTS} -c ${CONCURRENCY} \
    --latency-correction \
    --disable-keepalive \
    -m POST \
    -H "Content-Type: application/json" \
    -d '{"latency_ms": 50, "error_rate": 0.01}' \
    "$URL"

echo ""
echo "=== Stability Test Complete ==="
echo "Completed at: $(date)"
echo ""
echo "Assessment criteria:"
echo "- No crashes or panics"
echo "- Stable memory usage"
echo "- Reasonable CPU usage (depends on hardware)"
echo "- Distribution accuracy maintained"
echo ""
echo "Note: Results are from WSL2 Ubuntu (12 cores/24 threads, fast memory)."
echo "      Docker deployment may show different characteristics."
