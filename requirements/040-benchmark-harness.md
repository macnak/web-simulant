# Benchmark Harness

Purpose: validate stack decision (Rust engine + control plane) before full Phase 1 implementation.

## Objectives

- Confirm latency distribution fidelity under concurrent load
- Measure p50/p95/p99 latency and throughput
- Validate 8-12 hour soak stability
- Measure CPU and memory efficiency
- Compare against acceptance criteria (10% distribution accuracy, no crashes)

## Harness Structure

### Minimal Engine Implementation

Build lightweight Rust HTTP server with:

- Tokio async runtime
- Axum or Actix-Web for HTTP
- Single endpoint: POST /simulate
- Request body: `{"latency_ms": 50, "error_rate": 0.0}`
- Behavior: sleep for `latency_ms`, return 200 or 500 based on `error_rate`
- No UI, no configuration persistence (hardcoded for benchmark)

### Test Client

Use existing load testing tool:

- Option 1: `wrk` with Lua scripting for distributions
- Option 2: `oha` (Rust-based, simpler)
- Option 3: custom Rust client with precise distribution control

Client should:

- Generate requests according to workload profile
- Measure actual latency distribution
- Compare to expected distribution
- Report distribution error percentage

## Workloads

From requirements/030-stack-evaluation.md section 9:

1. Baseline: no artificial delay (measure natural overhead)
2. Fixed latency: 10 ms, 50 ms, 100 ms
3. Gaussian: mean 50 ms, std dev 10 ms
4. Error injection: 1%, 5%, 10% error rates
5. Long-run degradation: gradual latency increase over 8-12 hours

## Load Profiles

- Low: 10 concurrent, ~100 requests total
- Moderate: 1k concurrent, 1k TPS, 5 minutes
- High: 5k concurrent, stress TPS, 10 minutes
- Soak: 1k concurrent, steady TPS, 8-12 hours

## Metrics to Capture

- Request latency: p50, p95, p99, max
- Throughput: requests per second
- Error rate: actual vs expected
- CPU usage: average, peak, >80% duration
- Memory usage: RSS, peak
- Distribution error: |actual - expected| / expected

## Pass Criteria

From requirements:

- Distribution accuracy within 10% at >=100 samples
- No crashes during 8-12 hour soak
- CPU warnings logged when >80% for 10s
- Memory stable (no unbounded growth)

## Implementation Plan

### Step 1: Minimal Engine (1-2 days)

- Cargo project: `web-simulant-bench`
- Dependencies: tokio, axum (or actix-web), serde, serde_json
- Single endpoint with configurable delay and error rate
- Basic logging (env_logger)
- Docker build (optional, test locally first)

### Step 2: Workload Scripts (1 day)

- Shell scripts to run `wrk` or `oha` with different workload parameters
- Parse output and calculate distribution error
- Report pass/fail against criteria

### Step 3: Run Benchmarks (2-3 days)

- Execute all workload + load profile combinations
- Record results in evidence log (requirements/030-stack-evaluation.md)
- Document any issues or surprises

### Step 4: Soak Test (8-12 hours + monitoring)

- Run moderate load profile for 8-12 hours
- Monitor CPU, memory, logs
- Confirm no crashes, no memory leaks, no accuracy drift

### Step 5: Evidence Review (1 day)

- Update requirements/030-stack-evaluation.md with actual results
- Confirm Rust meets all pass criteria
- Document any concerns or adjustments needed for Phase 1

## Success Signal

Rust engine passes all workloads and soak test with:

- <10% distribution error at moderate/high load
- No crashes or panics during soak
- CPU <80% under moderate load (or warnings logged)
- Clear path to Phase 1 implementation

## Fallback Plan

If Rust fails any critical criteria:

- Re-evaluate Go as engine alternative
- Identify specific failure mode (latency fidelity, stability, CPU)
- Determine if issue is fixable (algorithm, library choice) or fundamental
- Update stack decision in requirements/030-stack-evaluation.md

## Timeline

- Week 1: implement minimal engine and workload scripts
- Week 2: run benchmarks and soak test
- Week 3: evidence review and Phase 1 kickoff (if pass)

Total: ~3 weeks before Phase 1 implementation begins.
