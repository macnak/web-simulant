# Web Simulant API Reference v1.0.0

## Overview

Web Simulant exposes two independent HTTP APIs:

- **Control Plane** (port 8081): Configuration and status management
- **Engine** (port 8080): Simulated API endpoints (dynamically loaded from config)

## Control Plane API (Port 8081)

### Health & Status Endpoints

#### GET /api/health

Check if control plane is operational.

**Response (200 OK)**:

```json
{
  "status": "ok"
}
```

---

#### GET /api/status

Get current system status including active endpoint count.

**Response (200 OK)**:

```json
{
  "status": "ok",
  "endpoints_active": 5,
  "config_loaded": true,
  "uptime_seconds": 1234
}
```

---

### Configuration Endpoints

#### POST /api/config/import

Upload and immediately apply a new configuration. Supports YAML, JSON, or multipart form submission.

**Content Types**:

- `application/json` - JSON configuration
- `application/yaml` - YAML configuration
- `multipart/form-data` - File upload

**Request (JSON)**:

```json
{
  "version": "1.0",
  "endpoints": [
    {
      "id": "get-users",
      "method": "GET",
      "path": "/api/users",
      "latency": { "distribution": "fixed", "params": { "delay_ms": 100 } },
      "response": {
        "status": 200,
        "body": "{\"users\": []}"
      },
      "error_profile": { "rate": 0.0 }
    }
  ]
}
```

**Request (YAML)**:

```yaml
version: "1.0"
endpoints:
  - id: get-users
    method: GET
    path: /api/users
    latency:
      distribution: fixed
      params:
        delay_ms: 100
    response:
      status: 200
      body: '{"users": []}'
    error_profile:
      rate: 0.0
```

**Response (200 OK) - Success**:

```json
{
  "status": "success",
  "summary": {
    "endpoints_loaded": 5,
    "warnings": []
  }
}
```

**Response (400 Bad Request) - Validation Error**:

```json
{
  "status": "error",
  "errors": ["Endpoint path must not be empty", "Latency mean must be positive"]
}
```

---

#### GET /api/config/export

Download currently loaded configuration in specified format.

**Query Parameters**:

- `format` (optional): `yaml` (default) or `json`

**Response (200 OK)**:

```yaml
version: "1.0"
endpoints:
  - method: GET
    path: /api/users
    ...
```

**Headers**:

- `Content-Type: application/yaml` or `application/json`
- `Content-Disposition: attachment; filename="web-simulant-config-<timestamp>.yaml"`

---

#### POST /api/config/validate

Validate a configuration without applying it. Useful for dry-run testing.

**Request (same as /api/config/import)**:

```json
{
  "version": "1.0",
  "endpoints": [...]
}
```

**Response (200 OK) - Valid**:

```json
{
  "status": "success",
  "summary": {
    "endpoints_loaded": 5,
    "warnings": ["Exponential distribution with mean < 10ms may be unstable"]
  }
}
```

**Response (400 Bad Request) - Invalid**:

```json
{
  "status": "error",
  "errors": [
    "Missing required field: endpoints",
    "Distribution type 'poisson' not supported"
  ]
}
```

---

### Endpoint Query Endpoints

#### GET /api/endpoints

List all currently active endpoints.

**Response (200 OK)**:

```json
{
  "endpoints_count": 2,
  "endpoints": [
    {
      "id": "get-users",
      "method": "GET",
      "path": "/api/users",
      "latency": {
        "distribution": "normal",
        "params": { "mean_ms": 100, "stddev_ms": 20 }
      },
      "error_rate": 0.02,
      "response_status": 200
    },
    {
      "id": "create-user",
      "method": "POST",
      "path": "/api/users",
      "latency": {
        "distribution": "fixed",
        "params": { "delay_ms": 50 }
      },
      "error_rate": 0.05,
      "response_status": 201
    }
  ]
}
```

---

#### GET /api/endpoints/{id}

Get details for a specific endpoint.

**Path Parameters**:

- `id`: Endpoint ID (e.g., `get-users`)

**Response (200 OK)**:

```json
{
  "id": "get-users",
  "method": "GET",
  "path": "/api/users",
  "latency": {
    "distribution": "normal",
    "params": { "mean_ms": 100, "stddev_ms": 20 }
  },
  "response": {
    "status": 200,
    "body": "{\"users\": []}"
  },
  "error_profile": {
    "rate": 0.02,
    "codes": [500]
  }
}
```

**Response (404 Not Found)**:

```json
{
  "error": "Endpoint not found",
  "requested_id": "DELETE-/api/users"
}
```

---

### Web UI

#### GET /

Serves the Web Simulant administration dashboard.

**Response**: HTML page with configuration interface, endpoint list, and status display.

---

## Engine API (Port 8080)

### Simulated Endpoints

All endpoints active in the loaded configuration are available on the engine port.

#### Health Check

Most configurations include a `/health` endpoint:

**Request**:

```
GET /health HTTP/1.1
Host: localhost:8080
```

**Response (200 OK)**:

```json
{
  "status": "ok"
}
```

#### Example: User API

Assuming configuration from above is loaded:

**Request**:

```
GET /api/users HTTP/1.1
Host: localhost:8080
```

**Response (200 OK - 95% of requests)**:

```json
{
  "users": [
    { "id": 1, "name": "Alice" },
    { "id": 2, "name": "Bob" }
  ]
}
```

**Response (503 Service Unavailable - 5% of requests)**:

```json
{
  "error": "Service temporarily unavailable",
  "retry_after": 30
}
```

**Behavior**:

- Latency: Normal distribution (100ms mean ± 20ms std dev)
- Success rate: 95% (5% error injection)
- Request body matching: None (accepts all POST/PUT bodies)

---

## Configuration Schema

### Top-Level Object

```typescript
{
  version: "1.0",              // Required: config version
  metadata?: Metadata,         // Optional: descriptive metadata
  endpoints: Endpoint[],        // Required: array of endpoint definitions
  endpoint_groups?: Group[],    // Optional: endpoint group definitions
  behavior_windows?: Window[],  // Optional: scoped behavior windows
  burst_events?: BurstEvent[],  // Optional: scoped burst events
  workflows?: Workflow[]        // Reserved for Phase 2 workflows
}
```

### Endpoint Object

```typescript
{
  id: "get-users",             // Required: unique identifier
  method: "GET",               // Required: HTTP method
  path: "/api/users",          // Required: URL path with leading /
  request?: RequestMatch,       // Optional: request body matching
  latency: Latency,             // Required: latency distribution config
  response: Response,           // Required: response template
  error_profile?: ErrorProfile, // Optional: error injection settings
  rate_limit?: RateLimit,       // Optional: per-endpoint rate limit
  bandwidth_cap?: BandwidthCap  // Optional: per-endpoint bandwidth cap
}
```

### Latency Object

```typescript
{
  distribution: "fixed" | "normal" | "exponential" | "uniform" | "log_normal" | "mixture",
  params: { ... } // Distribution-specific parameters
}
```

### Latency Profiles and Tuning

Use these profiles to shape response timing. The simulator samples a latency per request and sleeps for that duration. For distributions that can produce negative samples, values are clamped to 0 ms.

#### Fixed

- **Behavior**: Constant latency for every request.
- **Parameters**: `delay_ms` (integer, >= 0).
- **Use when**: You want deterministic timing or to validate client-side timeouts.
- **Tuning tip**: Set to your target steady-state latency (for example, 80-120 ms).

#### Normal (Gaussian)

- **Behavior**: Bell curve centered on `mean_ms` with symmetric variation.
- **Parameters**: `mean_ms` (integer, > 0), `stddev_ms` (integer, > 0).
- **Use when**: Latency clusters around a typical value with moderate jitter.
- **Tuning tips**:
  - About 68% of samples fall within 1 standard deviation and 95% within 2.
  - Keep `std_dev_ms` smaller than `mean_ms` to avoid frequent clamping to 0.

#### Exponential

- **Behavior**: Many fast responses with a long tail of slower ones.
- **Parameters**: `rate` (number, > 0). Use $rate = 1 / mean$.
- **Use when**: You want realistic tail latency or occasional spikes.
- **Tuning tips**:
  - Approximate percentiles: p50 ~ `mean_ms * 0.69`, p95 ~ `mean_ms * 3`, p99 ~ `mean_ms * 4.6`.
  - Avoid very small `mean_ms` values (< 10) if you expect meaningful tail latency.

#### Uniform

- **Behavior**: Flat distribution between `min_ms` and `max_ms`.
- **Parameters**: `min_ms` (integer, >= 0), `max_ms` (integer, >= min_ms).
- **Use when**: You want bounded jitter without a central peak.
- **Tuning tip**: Set `min_ms` to your baseline latency and `max_ms` to your worst-case bound.

#### Log-normal

- **Behavior**: Right-skewed with a long tail of slow responses.
- **Parameters**: `mean_ms` (integer, > 0), `stddev_ms` (integer, >= 0).
- **Use when**: You want realistic tail latency without a hard upper bound.
- **Tuning tip**: Increase `stddev_ms` to make the tail heavier.

#### Mixture

- **Behavior**: Weighted blend of multiple distributions (bimodal or multi-modal).
- **Parameters**: `components` array with `weight`, `distribution`, and `params`.
- **Use when**: You want cache hit/miss style behavior.
- **Tuning tip**: Weights are relative; they do not need to sum to 1.0.

#### Choosing a Profile

- **Deterministic SLO checks**: Fixed
- **Stable services with jitter**: Normal
- **Tail-latency modeling**: Exponential
- **Hard bounds**: Uniform

#### Fixed

```json
{
  "distribution": "fixed",
  "params": { "delay_ms": 100 }
}
```

#### Normal (Gaussian)

```json
{
  "distribution": "normal",
  "params": { "mean_ms": 100, "stddev_ms": 20 }
}
```

#### Exponential

```json
{
  "distribution": "exponential",
  "params": { "rate": 0.02 }
}
```

#### Uniform

```json
{
  "distribution": "uniform",
  "params": { "min_ms": 50, "max_ms": 150 }
}
```

#### Log-normal

```json
{
  "distribution": "log_normal",
  "params": { "mean_ms": 150, "stddev_ms": 60 }
}
```

#### Mixture

```json
{
  "distribution": "mixture",
  "params": {
    "components": [
      {
        "weight": 0.8,
        "distribution": "fixed",
        "params": { "delay_ms": 20 }
      },
      {
        "weight": 0.2,
        "distribution": "log_normal",
        "params": { "mean_ms": 250, "stddev_ms": 80 }
      }
    ]
  }
}
```

### Response Object

```typescript
{
  status: 200,                           // HTTP status code
  headers?: { [key: string]: string },   // Optional headers
  body: "{\"users\": []}"              // JSON response body (as string)
}
```

### ErrorProfile Object

```typescript
{
  rate: 0.05,                // Error probability (0.0-1.0)
  codes: [500, 503],          // Error status codes
  body?: "{...}",            // Error response body
  error_in_payload?: false,   // If true, error body returned with HTTP 200
  payload_corruption?: {      // Optional corruption settings
    rate: 0.2,
    mode: "truncate" | "replace",
    truncate_ratio?: 0.4,
    replacement?: "{...}"
  }
}
```

### RequestMatch Object

```typescript
{
  body_match: "any" | "exact" | "contains" | "ignore",
  body?: string
}
```

### EndpointGroup Object

```typescript
{
  id: "core-apis",
  endpoint_ids: ["get-users", "create-user"]
}
```

### BehaviorWindow Object

```typescript
{
  id?: "peak-load",
  scope: { endpoint_id?: string, group_id?: string, global?: boolean },
  schedule: {
    mode: "fixed" | "recurring",
    start_offset_ms?: number,
    duration_ms: number,
    every_ms?: number,
    jitter_ms?: number,
    max_occurrences?: number,
    min_delay_ms?: number
  },
  ramp?: { up_ms?: number, down_ms?: number, curve?: "linear" | "s_curve" },
  error_mix?: "override" | "additive" | "blend",
  latency_override?: Latency,
  error_profile_override?: ErrorProfile
}
```

### BurstEvent Object

```typescript
{
  id?: "error-spike",
  scope: { endpoint_id?: string, group_id?: string, global?: boolean },
  frequency: { every_ms: number, jitter_ms?: number },
  duration_ms: number,
  ramp?: { up_ms?: number, down_ms?: number, curve?: "linear" | "s_curve" },
  latency_spike?: Latency,
  error_spike?: { error_mix?: "override" | "additive" | "blend", error_profile: ErrorProfile }
}
```

---

## Error Responses

All error responses follow this format:

```json
{
  "status": "error",
  "errors": ["Error message 1", "Error message 2"],
  "timestamp": "2026-02-11T12:34:56Z"
}
```

### Common Errors

| Status | Error                                   | Cause                                 |
| ------ | --------------------------------------- | ------------------------------------- |
| 400    | `Invalid JSON`                          | Request body is not valid JSON        |
| 400    | `Missing required field: endpoints`     | Configuration missing endpoints array |
| 400    | `Endpoint path must not be empty`       | Endpoint path is blank or missing     |
| 400    | `Distribution type 'xyz' not supported` | Unknown latency distribution type     |
| 404    | `Endpoint not found`                    | Requested endpoint doesn't exist      |
| 415    | `Unsupported content type`              | Content-Type not supported            |
| 500    | `Internal server error`                 | Unexpected server error               |

---

## Rate Limiting & Quotas

Web Simulant supports optional per-endpoint rate limits and bandwidth caps via configuration.

**Rate limit**:

```yaml
rate_limit:
  requests_per_second: 5
  burst: 2
```

**Bandwidth cap**:

```yaml
bandwidth_cap:
  bytes_per_second: 10240
```

---

## Authentication

No authentication required for v1.0. All endpoints are publicly accessible on the network.

---

## Examples

### cURL: Upload Config

```bash
curl -X POST http://localhost:8081/api/config/import \
  -H "Content-Type: application/yaml" \
  --data-binary @config.yaml
```

### cURL: Test Simulated Endpoint

```bash
curl http://localhost:8080/api/users
```

### cURL: Export Current Config

```bash
curl http://localhost:8081/api/config/export?format=json > current-config.json
```

### Python: Upload Config

```python
import requests

config = {
    "version": "1.0",
    "endpoints": [{
    "id": "test",
    "method": "GET",
    "path": "/api/test",
    "latency": {"distribution": "fixed", "params": {"delay_ms": 100}},
    "response": {"status": 200, "body": '{"ok": true}'},
    "error_profile": {"rate": 0.0}
    }]
}

response = requests.post(
    'http://localhost:8081/api/config/import',
    json=config
)
print(response.json())
```

---

## Changelog

### v1.0.0 (2026-02-11)

- Initial release
- Control plane API with config management
- Engine with 4 latency distributions
- Web UI dashboard
- Docker deployment
- 46 unit and integration tests

---

_For more information, see [README.md](README.md) and [PROJECT-STATUS.md](PROJECT-STATUS.md)_
