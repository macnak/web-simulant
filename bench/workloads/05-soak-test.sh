#!/bin/bash
# Stability test: configurable duration at moderate load
# Default: 60 minutes at 1k TPS for realistic testing

set -e

SERVER_URL="${SERVER_URL:-http://localhost:8080}"
ENDPOINT="${ENDPOINT:-/simulate}"
URL="${SERVER_URL}${ENDPOINT}"
DURATION_MINUTES="${DURATION_MINUTES:-60}"
TPS="${TPS:-1000}"
CONCURRENCY="${CONCURRENCY:-1000}"

echo "=== Stability Test ==="
echo "Duration: ${DURATION_MINUTES} minutes"
echo "Target TPS: ${TPS}"
echo "Concurrency: ${CONCURRENCY}"
echo "Latency: 50ms"
echo "Error rate: 1%"
echo ""
echo "Note: This test is indicative only. Results may differ significantly"
echo "      in Docker or on different hardware. Current environment may be WSL2."
echo ""
echo "Starting at: $(date)"
echo ""

# Calculate total requests for duration
TOTAL_REQUESTS=$((TPS * 60 * DURATION_MINUTES))

echo "Total requests to send: ${TOTAL_REQUESTS}"
echo "Estimated completion: $(date -d "+${DURATION_MINUTES} minutes")"
echo ""
echo "Monitor resources in another terminal:"
echo "  watch -n 2 'ps aux | grep web-simulant-bench | grep -v grep'"
echo ""

# Run the test
# Note: oha rate limiting is approximate
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
echo "Pass criteria:"
echo "- No crashes or panics"
echo "- No memory leaks (stable RSS)"
echo "- Distribution accuracy maintained"
echo "- Reasonable CPU usage for hardware"
echo ""
echo "Important: These results are environment-specific and indicative only."
echo "Production behavior in Docker may differ significantly."
