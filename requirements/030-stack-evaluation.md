# Technology Stack Evaluation

This document scores candidate stacks against agreed criteria and weights.

## 1. Criteria and Weights

- Performance: 45
- Safety: 20
- Operability: 15
- Ecosystem: 10
- Development velocity: 5
- Hiring availability: 5

## 2. Minimum Thresholds

- Must support single executable or single Docker image distribution
- Must sustain 8 to 12 hour soak without failure
- Hot reload is optional for first release; required later
- Simple UI for configuration is required in first release; OpenAPI import is later

## 3. Candidate Stacks

Engine candidates:

- Rust
- Go
- Java
- C++

Control plane candidates:

- Node/TypeScript
- Go (HTMX/templating)
- Rust (Leptos/Actix)
- Python (FastAPI)

## 4. Scoring Matrix (Draft)

Scoring scale: 1 (poor) to 5 (excellent). Weighted totals to be calculated after evidence is collected.
Initial scores below are hypotheses that require benchmark validation.

| Candidate                      | Performance | Safety | Operability | Ecosystem | Dev velocity | Hiring | Weighted total | Evidence notes                                 |
| ------------------------------ | ----------- | ------ | ----------- | --------- | ------------ | ------ | -------------- | ---------------------------------------------- |
| Rust (engine)                  | 5           | 5      | 4           | 4         | 3            | 3      | 4.55           | High perf, memory safe, more complex dev.      |
| Go (engine)                    | 4           | 4      | 5           | 5         | 4            | 4      | 4.25           | Strong ops story, mature HTTP, good velocity.  |
| Java (engine)                  | 4           | 4      | 4           | 5         | 4            | 5      | 4.15           | Mature ecosystem, solid perf, heavier runtime. |
| C++ (engine)                   | 5           | 2      | 3           | 4         | 2            | 3      | 3.75           | Max perf potential, higher safety risk.        |
| Node/TS (control plane)        | 3           | 3      | 4           | 5         | 5            | 5      | 3.55           | Fast UI/dev, rich ecosystem.                   |
| Go (control plane)             | 4           | 4      | 5           | 4         | 4            | 4      | 4.15           | Simple deploy, solid perf, fewer UI libs.      |
| Rust (control plane)           | 4           | 5      | 4           | 3         | 2            | 3      | 3.95           | Safe and fast, slower UI iteration.            |
| Python/FastAPI (control plane) | 3           | 3      | 4           | 5         | 5            | 5      | 3.55           | High velocity, strong ecosystem, lower perf.   |

## 5. Summary (Hypothesis)

Based on current weighted totals, the leading engine candidate is Rust, with Go and Java close behind. For the control plane, Go is currently the top candidate by score, with Rust next.

Decision: single-stack Rust for both engine and control plane to maximize performance headroom, simplify deployment, and align with industry trends. UI iteration speed is secondary to engine performance.

## 6. Recommendations (Hypothesis)

- Engine: Rust (selected).
- Control plane: Rust (selected); single-stack approach chosen for simplicity and performance.
- Alternatives: Go (both layers) if team familiarity or UI velocity becomes a blocking concern during benchmarks.

## 7. Evidence Log

Add references, benchmarks, and rationale for each score.

- Initial scores are estimates based on typical language/runtime characteristics.
- Benchmark evidence is required before final selection.

Candidate evidence placeholders:

- Rust (engine):
  - **Evidence collected** (WSL2 Ubuntu, 12 cores/24 threads - indicative only):
    - Natural overhead: 1.5-2ms at moderate concurrency (100 concurrent)
    - Distribution accuracy: 3.6% overhead at 50ms (p50), 1.6% overhead at 100ms (p50)
    - High concurrency: 100% success rate at 10k concurrent (100k requests)
    - Error injection: 100% accurate (1%, 5%, 10% rates all exact)
    - Throughput: >100k req/sec (environment-specific, not production indicator)
    - Stability: No crashes during short-duration stress testing
  - **Evidence needed**: Medium-duration stability test (10-60 min) at moderate TPS; Docker environment validation.
  - **Rationale**: Strong performance and memory safety for engine core; predictable overhead allows accurate distribution shaping. Short-duration tests all passed. Note: WSL2 results not representative of production deployment.
- Go (engine): evidence needed - distribution accuracy vs throughput, goroutine overhead under 10k concurrency, soak stability; rationale - simple ops and good performance.
- Java (engine): evidence needed - GC impact on tail latency, soak stability, packaging into single image; rationale - mature ecosystem, heavier runtime.
- C++ (engine): evidence needed - safety risk mitigation, development velocity, soak stability; rationale - max perf potential, higher safety risk.
- Node/TS (control plane): evidence needed - UI speed and stability under configuration edits, packaging with engine; rationale - fast UI development and broad ecosystem.
- Go (control plane): evidence needed - UI/templating flexibility, packaging, operability; rationale - simple deploy, solid performance.
- Rust (control plane): evidence needed - UI iteration speed, developer productivity, packaging; rationale - strong safety, slower UI iteration.
- Python/FastAPI (control plane): evidence needed - throughput under admin/config usage, packaging; rationale - high velocity, lower perf.

## 7.1 Priority Evidence Checklist

Top priority (engine):

- Latency distribution fidelity at 5k to 10k concurrency
- Tail latency impact under sustained load (p95/p99)
- 8 to 12 hour soak stability
- CPU efficiency vs distribution accuracy drift

Top priority (control plane):

- UI configuration flow responsiveness
- Packaging into single Docker image with engine
- Admin API responsiveness under normal usage

## 8. Candidate Narrowing (Hypothesis)

- Engine shortlist: Rust, Go
- Control plane shortlist: Go, Node/TypeScript

## 9. Benchmark Plan (Draft)

Objectives:

- Validate latency distribution fidelity under load.
- Measure throughput, tail latency, and CPU/memory efficiency.
- Confirm 8 to 12 hour soak stability.
- Evaluate config reload behavior (later phase).

Workloads:

- Baseline: no artificial delay configured (measure natural overhead).
- Fixed latency: 10 ms, 50 ms, 100 ms.
- Gaussian latency: mean 50 ms, std dev 10 ms; verify distribution shape.
- Error injection: 1%, 5%, 10% error rates with clustering.
- Long-run degradation: gradual latency increase over time window.

Load profiles:

- Low TPS (sub-1 TPS) and low concurrency (1 to 10).
- Moderate load (1k concurrent, 1k TPS target).
- High load (5k to 10k concurrent, stress TPS).

Metrics to capture:

- p50/p95/p99 latency and distribution error vs config.
- Throughput (TPS) and success/error counts.
- CPU and memory usage over time.
- Stability during 8 to 12 hour soak.

Pass criteria (initial):

- Distribution accuracy within 10% for sample sizes >=100.
- No crashes or data corruption during soak test.
- CPU warning thresholds align with observed drift in accuracy.
