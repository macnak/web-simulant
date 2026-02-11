# Workload Test Scripts

Prerequisites:

- Server running on `http://localhost:8080` (or set `SERVER_URL` environment variable)
- `oha` installed: `cargo install oha` (Rust-based load testing tool)

Alternative: use `wrk` if `oha` is not available.

## Quick Start

```bash
# Make scripts executable
chmod +x *.sh

# Run all workloads (except soak test)
./run-all.sh

# Run individual workloads
./01-baseline.sh
./02-fixed-latency.sh
./03-error-injection.sh
./04-high-concurrency.sh

# Run soak test (8-12 hours)
DURATION_HOURS=8 ./05-soak-test.sh
```

## Workload Descriptions

### 01-baseline.sh

Measures natural overhead with 0ms configured latency:

- Low concurrency: 10 concurrent, 100 requests
- Moderate: 100 concurrent, 1000 requests
- High: 1000 concurrent, 10000 requests

### 02-fixed-latency.sh

Tests fixed delays (10ms, 50ms, 100ms):

- Validates distribution accuracy
- Expected p50/p95/p99 close to configured value
- Tolerance: within 10% at >=100 samples

### 03-error-injection.sh

Tests error rates (1%, 5%, 10%):

- 50ms base latency
- Validates error rate accuracy

### 04-high-concurrency.sh

Stress tests at high concurrency:

- 5k concurrent, 50k total requests
- 10k concurrent, 100k total requests
- Monitor CPU and memory during execution

### 05-soak-test.sh

Long-running stability test:

- Default: 8 hours (configurable via DURATION_HOURS)
- 1000 concurrent connections
- ~1000 TPS target
- Pass criteria: no crashes, stable memory, maintained accuracy

## Monitoring During Tests

```bash
# In another terminal, monitor server resources
watch -n 1 'ps aux | grep web-simulant-bench'

# Or use htop for interactive monitoring
htop -p $(pgrep web-simulant-bench)

# Check server logs
tail -f /path/to/server.log
```

## Pass Criteria

From requirements/030-stack-evaluation.md:

- Distribution accuracy within 10% at >=100 samples
- No crashes during stability tests
- CPU warnings logged when >80% for 10s
- Memory stable (no leaks or unbounded growth)

**Important**: Benchmark results are environment-specific and indicative only.

- Current tests run on WSL2 Ubuntu (12 cores/24 threads, fast memory)
- Docker deployment will show different performance characteristics
- Production throughput may vary significantly based on deployment environment

## Results Collection

Record results in: `requirements/030-stack-evaluation.md` section 7 (Evidence Log).

Key metrics to capture:

- p50/p95/p99 latencies
- Throughput (TPS)
- Error rates vs expected
- CPU/memory usage
- Any crashes or anomalies
