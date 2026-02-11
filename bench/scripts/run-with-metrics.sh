#!/bin/bash
# Enhanced server wrapper with metrics collection

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Configuration
METRICS_FILE="${METRICS_FILE:-metrics.log}"
METRICS_INTERVAL="${METRICS_INTERVAL:-5}"
PID_FILE="server.pid"

# Build if needed
if [ ! -f "${PROJECT_ROOT}/target/release/web-simulant-bench" ]; then
    echo "Building server..."
    cd "${PROJECT_ROOT}"
    cargo build --release
fi

# Start server in background
echo "Starting server..."
cd "${PROJECT_ROOT}"
RUST_LOG=info ./target/release/web-simulant-bench > server.log 2>&1 &
SERVER_PID=$!
echo ${SERVER_PID} > "${PID_FILE}"

echo "Server started with PID: ${SERVER_PID}"
echo "Logs: ${PROJECT_ROOT}/server.log"

# Wait for server to be ready
echo "Waiting for server to be ready..."
for i in {1..30}; do
    if curl -sf http://localhost:8080/health > /dev/null 2>&1; then
        echo "Server is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "Error: Server failed to start"
        kill ${SERVER_PID} 2>/dev/null || true
        exit 1
    fi
    sleep 1
done

# Start metrics collection in background
echo "Starting metrics collection (interval: ${METRICS_INTERVAL}s)..."
while kill -0 ${SERVER_PID} 2>/dev/null; do
    TIMESTAMP=$(date +%s)
    CPU=$(ps -p ${SERVER_PID} -o %cpu= 2>/dev/null || echo "0")
    MEM=$(ps -p ${SERVER_PID} -o rss= 2>/dev/null || echo "0")
    echo "${TIMESTAMP},${CPU},${MEM}" >> "${METRICS_FILE}"
    sleep ${METRICS_INTERVAL}
done &
METRICS_PID=$!

echo "Metrics collection started with PID: ${METRICS_PID}"
echo "Metrics file: ${PROJECT_ROOT}/${METRICS_FILE}"
echo ""
echo "Server is running. Press Ctrl+C to stop."
echo ""

# Cleanup handler
cleanup() {
    echo ""
    echo "Stopping server and metrics collection..."
    kill ${METRICS_PID} 2>/dev/null || true
    kill ${SERVER_PID} 2>/dev/null || true
    rm -f "${PID_FILE}"
    echo "Stopped."
}

trap cleanup EXIT INT TERM

# Wait for server process
wait ${SERVER_PID}
