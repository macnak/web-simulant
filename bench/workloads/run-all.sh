#!/bin/bash
# Run all workload tests in sequence

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "========================================="
echo "Web Simulant Benchmark Suite"
echo "========================================="
echo ""
echo "Server URL: ${SERVER_URL:-http://localhost:8080}"
echo ""

# Health check first
echo "Checking server health..."
curl -f "${SERVER_URL:-http://localhost:8080}/health" || {
    echo "Error: Server not responding"
    exit 1
}
echo "Server is healthy"
echo ""

# Run workloads
"${SCRIPT_DIR}/01-baseline.sh"
echo ""

"${SCRIPT_DIR}/02-fixed-latency.sh"
echo ""

"${SCRIPT_DIR}/03-error-injection.sh"
echo ""

"${SCRIPT_DIR}/04-high-concurrency.sh"
echo ""

echo "========================================="
echo "Benchmark Suite Complete"
echo "========================================="
echo ""
echo "Next steps:"
echo "1. Review results for distribution accuracy (<10% error)"
echo "2. Check for any crashes or errors"
echo "3. Run soak test separately: ./05-soak-test.sh"
echo ""
