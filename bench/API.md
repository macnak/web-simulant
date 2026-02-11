# API Documentation

## Phase 0 (Current) - Benchmark Harness

This is a minimal validation server for Phase 0 testing. Limited API surface for benchmarking only.

### Connection Details

- **Host**: `localhost` (or `0.0.0.0` when running in container)
- **Port**: `8080`
- **Base URL**: `http://localhost:8080`
- **Protocol**: HTTP (no HTTPS in Phase 0)

### Available Endpoints

#### Health Check

Check if the server is running.

```http
GET /health
```

**Response**: `200 OK`

```
OK
```

**Example**:

```bash
curl http://localhost:8080/health
```

#### Simulate Request

Simulate an endpoint with configurable latency and error behavior.

```http
POST /simulate
Content-Type: application/json
```

**Request Body**:

```json
{
  "latency_ms": 50,
  "error_rate": 0.05
}
```

**Parameters**:

- `latency_ms` (required): Delay in milliseconds before responding (0 to 10000+)
- `error_rate` (optional): Probability of error response, 0.0 to 1.0 (default: 0.0)
  - `0.0` = no errors
  - `0.05` = 5% error rate
  - `1.0` = 100% errors

**Success Response**: `200 OK`

```json
{
  "message": "Success",
  "latency_ms": 50
}
```

**Error Response**: `500 Internal Server Error`

```json
{
  "error": "Simulated error",
  "latency_ms": 50
}
```

**Examples**:

```bash
# No latency, no errors
curl -X POST http://localhost:8080/simulate \
  -H "Content-Type: application/json" \
  -d '{"latency_ms": 0, "error_rate": 0.0}'

# 50ms latency, 5% error rate
curl -X POST http://localhost:8080/simulate \
  -H "Content-Type: application/json" \
  -d '{"latency_ms": 50, "error_rate": 0.05}'

# 100ms latency, 10% error rate
curl -X POST http://localhost:8080/simulate \
  -H "Content-Type: application/json" \
  -d '{"latency_ms": 100, "error_rate": 0.10}'
```

**Error Behavior**:

- Error injection is deterministic based on request count
- With 5% error rate, approximately 5 out of every 100 requests will return 500
- Errors are distributed evenly across request stream

**Performance Characteristics**:

- **Natural overhead**: ~4ms measured (consistent across different configured latencies)
- **Accuracy**: Actual latency = configured latency + 4ms overhead
  - Example: 40ms configured → 44ms actual response time
  - At 100ms configured: 4% overhead (within 10% target)
  - At 40ms configured: 10% overhead (meets target threshold)
  - At 10ms configured: 40% overhead (exceeds target - use higher latencies for accuracy)
- **Recommendation**: For <10% accuracy, configure latencies ≥40ms

See [RESULTS.md](RESULTS.md) for detailed benchmark findings.

---

## Phase 1 (Planned) - Full Implementation

Phase 1 will add a control plane with UI and configuration APIs.

### Planned Ports

- **Engine Port**: `8080` - Simulation endpoints (same as Phase 0)
- **Control Plane Port**: `8081` - Configuration UI and Admin APIs

### Planned UI Access

The web-based configuration UI will be available at:

```
http://localhost:8081/
```

**UI Features** (Phase 1):

- Dashboard overview of configured endpoints
- Create/edit/delete endpoint configurations
- Configure latency distributions (Normal, Exponential, Uniform, Fixed)
- Configure error injection patterns
- Import/export configuration (YAML/JSON)
- View basic metrics

### Planned Control Plane APIs

Base URL: `http://localhost:8081`

#### Admin Endpoints

```http
GET /admin/health
```

Health check for control plane.

```http
GET /admin/metrics
```

Export current metrics in JSON format.

**Response**:

```json
{
  "uptime_seconds": 3600,
  "total_requests": 150000,
  "current_concurrency": 245,
  "endpoints": [
    {
      "id": "endpoint-1",
      "path": "/api/users",
      "method": "GET",
      "request_count": 50000,
      "success_count": 49500,
      "error_count": 500,
      "latency_p50_ms": 48.5,
      "latency_p95_ms": 52.3,
      "latency_p99_ms": 58.7
    }
  ]
}
```

#### Endpoint Configuration APIs

```http
GET /api/endpoints
```

List all configured endpoints.

```http
POST /api/endpoints
```

Create a new endpoint configuration.

**Request Body**:

```json
{
  "id": "users-get",
  "method": "GET",
  "path": "/api/users",
  "latency": {
    "distribution": "normal",
    "mean_ms": 50,
    "stddev_ms": 10
  },
  "error_profile": {
    "rate": 0.01,
    "codes": [500, 503]
  }
}
```

```http
GET /api/endpoints/:id
```

Get a specific endpoint configuration.

```http
PUT /api/endpoints/:id
```

Update an endpoint configuration.

```http
DELETE /api/endpoints/:id
```

Delete an endpoint configuration.

#### Workflow Management APIs

```http
GET /api/workflows
POST /api/workflows
GET /api/workflows/:id
PUT /api/workflows/:id
DELETE /api/workflows/:id
```

Manage workflow configurations (groups of endpoints with overrides).

#### Configuration Import/Export

```http
POST /api/config/import
```

Import configuration from YAML or JSON.

```http
GET /api/config/export
```

Export current configuration as YAML or JSON.

---

## Phase 2 (Future)

Additional control plane features:

- Configuration versioning and rollback
- Advanced workflow management UI
- Time-series metrics storage
- OpenAPI import for endpoint generation
- Scenario templates

---

## Testing the API

### Using curl

```bash
# Health check
curl http://localhost:8080/health

# Basic simulation
curl -X POST http://localhost:8080/simulate \
  -H "Content-Type: application/json" \
  -d '{"latency_ms": 50, "error_rate": 0.0}'

# With error injection
curl -X POST http://localhost:8080/simulate \
  -H "Content-Type: application/json" \
  -d '{"latency_ms": 100, "error_rate": 0.10}'
```

### Using httpie

```bash
# Health check
http GET localhost:8080/health

# Simulation
http POST localhost:8080/simulate \
  latency_ms:=50 \
  error_rate:=0.05
```

### Load Testing

See [workloads/README.md](workloads/README.md) for benchmark scripts using `oha`.

---

## Error Responses

### 404 Not Found

Endpoint does not exist.

```json
{
  "error": "Not Found"
}
```

### 400 Bad Request

Invalid request body or parameters.

```json
{
  "error": "Invalid request body"
}
```

### 500 Internal Server Error

Either simulated error (when configured) or actual server error.

---

## Notes

- **Phase 0**: Only `/health` and `/simulate` are available
- **Authentication**: None in Phase 0/1 (bind to localhost for security)
- **Rate Limiting**: Not implemented in Phase 0
- **CORS**: Not configured in Phase 0
- **Content-Type**: All POST requests require `application/json`

See [requirements/020-design.md](../requirements/020-design.md) for full architecture details.
