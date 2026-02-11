# Web Simulant - Docker Hub Description

## Short Description

Web API simulation engine for performance testing with configurable latency distributions, error injection, and realistic API degradation patterns.

---

## Full Description

**Web Simulant** is a high-performance API simulation platform built in Rust for realistic performance and chaos testing. It runs simulated endpoints with configurable latencies, error rates, and failure patterns—perfect for load testing, resilience testing, and API client validation.

### Key Features

- **Multiple Latency Distributions**: Fixed, Normal, Exponential, Uniform
- **Error Injection**: Configurable error rates with HTTP status codes
- **Request Body Matching**: Route based on request payload patterns
- **Web Dashboard**: Clean UI for config management and monitoring
- **Config Management**: Upload/download configs in YAML or JSON
- **High Concurrency**: Handles thousands of concurrent requests
- **Two-Port Architecture**:
  - **Port 8080**: Engine with simulated API endpoints
  - **Port 8081**: Control plane with admin UI and APIs

### Quick Start

```bash
# Using Docker Compose (recommended)
docker-compose up

# Or standalone Docker
docker run -p 8080:8080 -p 8081:8081 web-simulant:latest
```

Then:

- Open browser: http://localhost:8081/
- Test API: `curl http://localhost:8080/health`
- Upload config: Use web UI or `curl -X POST http://localhost:8081/api/config/import`

### Configuration

Supply configs via:

- **Web UI** at localhost:8081 (upload YAML/JSON)
- **Multipart form** POST /api/config/import
- **Startup** from config/active.yaml (persisted)
- **Examples** in /examples (4 ready-to-use templates)

### Example Config (YAML)

```yaml
version: "1.0"
endpoints:
  - method: GET
    path: /api/users
    latency:
      type: normal
      mean_ms: 100
      std_dev_ms: 20
    responses:
      - status: 200
        body: '{"users": [{"id": 1, "name": "Alice"}]}'
        error_rate: 0.02
```

### API Endpoints

**Control Plane (localhost:8081)**

- `GET /api/health` - Health check
- `GET /api/status` - System status
- `GET /api/endpoints` - List active endpoints
- `POST /api/config/import` - Upload and apply config
- `GET /api/config/export` - Download current config as YAML or JSON
- `POST /api/config/validate` - Validate config without applying
- `GET /` - Web UI dashboard

**Engine (localhost:8080)**

- Dynamic endpoints based on loaded config
- See examples for endpoint patterns

### Data Persistence

Configs are automatically saved to `/app/config/active.yaml` inside container. Mount this directory to persist across restarts:

```bash
docker run -p 8080:8080 -p 8081:8081 \
  -v /path/to/config:/app/config \
  web-simulant:latest
```

### Environment Variables

- `RUST_LOG` - Logging level (default: `info`)
  - Options: `trace`, `debug`, `info`, `warn`, `error`

Example:

```bash
docker run -e RUST_LOG=debug -p 8080:8080 -p 8081:8081 web-simulant:latest
```

### Supported Distributions

1. **Fixed**: Constant latency
2. **Normal**: Gaussian distribution with mean and standard deviation
3. **Exponential**: Exponential decay (realistic for network latency)
4. **Uniform**: Random within min/max range

### Error Handling

- Configurable error rates per endpoint (0.0–1.0)
- Custom HTTP status codes (4xx, 5xx)
- Request body matching for conditional failures
- Realistic error response bodies

### Use Cases

✓ Load testing against realistic latency profiles  
✓ Chaos engineering and resilience testing  
✓ Client library validation  
✓ Performance regression testing  
✓ Network degradation simulation  
✓ Error handling verification

### Testing

The image includes an integration test suite:

```bash
docker run web-simulant:latest /bin/sh -c "cargo test --release"
```

All 46 unit and integration tests pass.

### Resource Requirements

- **CPU**: 100m–500m (depends on load)
- **Memory**: 50MB–200MB (depends on concurrency)
- **Image Size**: ~120MB (Alpine Linux base)

### Health Checks

Container includes HTTP health check on `/api/health`. Docker Compose automatically monitors this with 10-second intervals.

### License

MIT

### Support & Contributions

For issues, questions, or contributions: [GitHub](https://github.com/yourusername/web-simulant)

### Tags

`latest`, `v1.0.0`, `stable`

### Source

Built from: https://github.com/yourusername/web-simulant

---

_Web Simulant v1.0.0 - Performance Testing Made Simple_
