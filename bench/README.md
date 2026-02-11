# Web Simulant Benchmark Harness

Phase 0 validation harness to test Rust engine performance before full implementation.

## Build and Run

```bash
# Build release binary
cargo build --release

# Run server
RUST_LOG=info ./target/release/web-simulant-bench

# Or run directly with cargo
RUST_LOG=info cargo run --release
```

Server listens on `0.0.0.0:8080`.

## Endpoints

- `GET /health` - Health check (returns 200 OK)
- `POST /simulate` - Simulate request with configured behavior

### Simulate Request

```json
{
  "latency_ms": 50,
  "error_rate": 0.05
}
```

Parameters:

- `latency_ms`: delay in milliseconds before responding
- `error_rate`: probability of error response (0.0 to 1.0, optional, default 0.0)

Success response (200):

```json
{
  "message": "Success",
  "latency_ms": 50
}
```

Error response (500):

```json
{
  "error": "Simulated error",
  "latency_ms": 50
}
```

## Quick Test

```bash
# Check server is running
curl http://localhost:8080/health

# Test simulate endpoint
curl -X POST http://localhost:8080/simulate \
  -H "Content-Type: application/json" \
  -d '{"latency_ms": 50, "error_rate": 0.05}'
```

See [API.md](API.md) for complete endpoint documentation.

## Performance Characteristics

**Measured overhead**: ~4ms (consistent)

- 40ms configured → 44ms actual (10% overhead)
- 100ms configured → 104ms actual (4% overhead)
- Recommendation: Use latencies ≥40ms for <10% accuracy

See [RESULTS.md](RESULTS.md) for detailed benchmark results.

## Performance Targets

- Distribution accuracy: within 10% at >=100 samples (✓ validated at ≥40ms)
- Concurrency: up to 10k concurrent requests
- Soak stability: 8-12 hours without crash
- CPU efficiency: <80% under moderate load
