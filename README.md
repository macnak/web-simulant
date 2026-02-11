# Web Simulant

Web API simulation application for performance testing. Simulates realistic API behaviors with configurable latency distributions, error injection, and degradation patterns.

## Current Status: Phase 0 - Benchmark Validation

Testing Rust stack performance before full implementation.

## Quick Links

- **[Quick Reference](QUICK-REFERENCE.md)** - Server connection & API endpoints
- **[API Documentation](bench/API.md)** - Complete endpoint reference
- **[Benchmark Harness](bench/README.md)** - Phase 0 testing
- **[Requirements](requirements/000-overview.md)** - Project overview
- **[Design](requirements/020-design.md)** - Architecture details

## Connection Details

### Phase 0 (Current)

- **Server**: `http://localhost:8080`
- **Endpoints**: `/health`, `/simulate`
- **UI**: Not available (command-line only)

### Phase 1 (Planned)

- **Engine**: `http://localhost:8080` - Simulated API endpoints
- **Control Plane**: `http://localhost:8081` - Admin & Config
- **Web UI**: `http://localhost:8081/` - Dashboard

## Quick Start

```bash
# Run benchmark server
cd bench
cargo run --release

# Test it works
curl http://localhost:8080/health
curl -X POST http://localhost:8080/simulate \
  -H "Content-Type: application/json" \
  -d '{"latency_ms": 50, "error_rate": 0.05}'

# Run benchmark suite
cd workloads
./run-all.sh
```

## Features

### Current (Phase 0)

- ✓ Configurable latency simulation
- ✓ Error injection with configurable rates
- ✓ High concurrency support (up to 10k)
- ✓ Benchmark workload scripts
- ✓ CPU/memory metrics collection

### Coming (Phase 1)

- Web UI dashboard
- Multiple latency distributions (Normal, Exponential, Uniform, Fixed)
- Endpoint configuration management
- Workflow and scenario support
- Config import/export (YAML/JSON)
- Real-time metrics display
- 8-12 hour soak stability

## Documentation Structure

```
requirements/
├── 000-overview.md          # Project overview and goals
├── 001-phased-plan.md       # Delivery phases
├── 010-requirements.md      # Functional/non-functional requirements
├── 020-design.md            # Architecture and design decisions
├── 030-stack-evaluation.md  # Technology stack evaluation
├── 040-benchmark-harness.md # Benchmark plan
└── 990-project-memory.md    # Project decisions log

bench/
├── src/main.rs              # Minimal benchmark server
├── workloads/               # Test scripts
├── scripts/                 # Metrics tools
└── API.md                   # API documentation
```

## Technology Stack

**Decision**: Single-stack Rust for both engine and control plane

- **Performance**: Maximum concurrency and efficiency
- **Safety**: Memory safety without garbage collection
- **Deployment**: Single binary, simple Docker image
- **Runtime**: Tokio async with Axum HTTP framework

## Goals

Build a performance testing tool that:

- Simulates realistic API behaviors (latency, errors, degradation)
- Handles 8-12 hour soak tests without failure
- Supports up to 10k concurrent requests
- Provides simple UI for configuration
- Deploys as single executable or Docker image

## License

TBD

## Contributing

Currently in initial development phase.
