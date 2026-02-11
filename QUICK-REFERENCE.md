# Quick Reference - Server Connection & Endpoints

## Phase 0 (Current - Benchmark Harness)

### Connection

- **URL**: `http://localhost:8080`
- **No UI**: Command-line/API only

### Endpoints

```bash
# Health check
curl http://localhost:8080/health

# Simulate with 50ms latency, 5% errors
curl -X POST http://localhost:8080/simulate \
  -H "Content-Type: application/json" \
  -d '{"latency_ms": 50, "error_rate": 0.05}'
```

See [API.md](bench/API.md) for full API documentation.

---

## Phase 1 (Planned)

### Connection

- **Engine**: `http://localhost:8080` - Simulation endpoints
- **Control Plane**: `http://localhost:8081` - Configuration & Admin
- **Web UI**: `http://localhost:8081/` - Dashboard

### Port Separation

- Port 8080: Handles simulated API traffic (your test requests)
- Port 8081: Manages configuration and displays metrics (admin interface)

### UI Features

- Visual dashboard to create/edit endpoints
- Configure latency distributions and error injection
- Import/export configurations
- View real-time metrics

### Planned API Endpoints

**Engine (8080)**:

- User-configured endpoints (e.g., `/api/users`, `/api/products`)
- Behavior determined by configuration

**Control Plane (8081)**:

- `GET /` - Web UI dashboard
- `GET /admin/health` - Health check
- `GET /admin/metrics` - Metrics export
- `POST /api/endpoints` - Create/update endpoint config
- `POST /api/config/import` - Import YAML/JSON

---

## Documentation

- **API Details**: [bench/API.md](bench/API.md)
- **Architecture**: [requirements/020-design.md](requirements/020-design.md)
- **Requirements**: [requirements/000-overview.md](requirements/000-overview.md)
- **Benchmark Tests**: [bench/workloads/README.md](bench/workloads/README.md)

---

## Quick Start

### Run Phase 0 Server

```bash
cd bench
cargo run --release
```

### Test Endpoints

```bash
# Check health
curl http://localhost:8080/health

# Run benchmark suite
cd workloads
./run-all.sh
```

### Run with Metrics

```bash
cd bench/scripts
./run-with-metrics.sh
```
