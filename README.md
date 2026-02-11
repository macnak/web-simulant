# Web Simulant

Web API simulation application for performance testing. Simulates realistic API behaviors with configurable latency distributions, error injection, and degradation patterns.

## Current Status: v1.0.0 Release ✓

**Phase 1.1–1.12 Complete** - Engine, control plane API, UI, Docker deployment, and full documentation delivered.

**Download**: [Docker Hub](https://hub.docker.com/r/yourusername/web-simulant:v1.0.0) | [GitHub Releases](https://github.com/yourusername/web-simulant/releases/tag/v1.0.0)

## Quick Links

- **[PROJECT-STATUS.md](PROJECT-STATUS.md)** ← Current state & daily progress
- **[Quick Reference](QUICK-REFERENCE.md)** - Server connection & API endpoints
- **[Configuration Schema](requirements/050-configuration-schema.md)** - APPROVED schema definition
- **[Example Configs](examples/)** - 4 complete example configurations
- **[API Documentation](bench/API.md)** - Complete endpoint reference
- **[Benchmark Results](bench/RESULTS.md)** - Phase 0 validation results
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

### Option 1: Docker Compose (Recommended)

```bash
# Start both engine and control plane
docker-compose up --build

# In another terminal, test the API
curl http://localhost:8081/api/health
curl http://localhost:8080/health

# Open UI in browser
# http://localhost:8081/

# Stop
docker-compose down
```

### Option 2: Docker (Manual)

```bash
# Build image
docker build -t web-simulant .

# Run container
docker run -p 8080:8080 -p 8081:8081 \
  -v $(pwd)/config:/app/config \
  -v $(pwd)/examples:/app/examples \
  web-simulant

# Test
curl http://localhost:8081/api/health
```

### Option 3: Native Build

```bash
# Build release binary
cargo build --release

# Run server
./target/release/web-simulant

# In another terminal, test it
curl http://localhost:8081/api/health
curl http://localhost:8080/health

# Open UI in browser
# http://localhost:8081/
```

### API Endpoints

**Control Plane (Admin & Config)** – http://localhost:8081

- `GET /api/health` – Health check
- `GET /api/status` – Current status
- `GET /api/endpoints` – List active endpoints
- `POST /api/config/import` – Upload YAML/JSON config
- `GET /api/config/export` – Download current config
- `POST /api/config/validate` – Validate config without applying

**Engine (Simulated APIs)** – http://localhost:8080

- Dynamic endpoints based on loaded config
- See [Example Configs](examples/) for endpoint samples

**Web UI** – http://localhost:8081/

- Visual dashboard for config management
- Real-time endpoint monitoring
- Config import/export interface

## Features

### Phase 1 (Complete ✓)

- ✓ Web UI dashboard with config management
- ✓ Multiple latency distributions (Normal, Exponential, Uniform, Fixed)
- ✓ Full endpoint configuration management
- ✓ Workflow and scenario support
- ✓ Config import/export (YAML/JSON, multipart upload)
- ✓ Real-time endpoints list and status
- ✓ Error injection with configurable rates
- ✓ Integration testing suite
- ✓ Docker deployment (Dockerfile + docker-compose)
- ✓ Persistence (atomic config saves)

### Phase 2 (Planned)

- UI endpoint editor (add/edit/delete endpoints)
- Advanced metrics collection and graphing
- Load profile simulation
- Webhook callbacks
- API rate limiting
- Advanced scenario workflows

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
