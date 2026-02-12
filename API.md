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
      "method": "GET",
      "path": "/api/users",
      "latency": { "type": "fixed", "value_ms": 100 },
      "responses": [
        {
          "status": 200,
          "body": "{\"users\": []}",
          "error_rate": 0.0
        }
      ]
    }
  ]
}
```

**Request (YAML)**:

```yaml
version: "1.0"
endpoints:
  - method: GET
    path: /api/users
    latency:
      type: fixed
      value_ms: 100
    responses:
      - status: 200
        body: '{"users": []}'
        error_rate: 0.0
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
  "endpoints": [
    {
      "id": "GET-/api/users",
      "method": "GET",
      "path": "/api/users",
      "latency_type": "normal",
      "error_rate": 0.02
    },
    {
      "id": "POST-/api/users",
      "method": "POST",
      "path": "/api/users",
      "latency_type": "fixed",
      "error_rate": 0.05
    }
  ]
}
```

---

#### GET /api/endpoints/{id}

Get details for a specific endpoint.

**Path Parameters**:

- `id`: Endpoint ID in format `METHOD-path` (e.g., `GET-/api/users`)

**Response (200 OK)**:

```json
{
  "id": "GET-/api/users",
  "method": "GET",
  "path": "/api/users",
  "latency": {
    "type": "normal",
    "mean_ms": 100,
    "std_dev_ms": 20
  },
  "responses": [
    {
      "status": 200,
      "body": "{\"users\": []}",
      "error_rate": 0.02
    }
  ]
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
  version: "1.0",           // Required: config version
  endpoints: Endpoint[]     // Required: array of endpoint definitions
}
```

### Endpoint Object

```typescript
{
  method: "GET"           // Required: HTTP method (GET, POST, PUT, DELETE, etc.)
  path: "/api/users"      // Required: URL path with leading /
  latency: Latency        // Required: latency distribution config
  responses: Response[]   // Required: response templates
  request_match?: {       // Optional: match request body patterns
    type: "json_path"     // Comparison type
    pattern: "$.user_id"  // Pattern to match
  }
}
```

### Latency Object

All distributions support these fields:

### Latency Profiles and Tuning

Use these profiles to shape response timing. The simulator samples a latency per request and sleeps for that duration. For distributions that can produce negative samples, values are clamped to 0 ms.

#### Fixed

- **Behavior**: Constant latency for every request.
- **Parameters**: `value_ms` (integer, >= 0).
- **Use when**: You want deterministic timing or to validate client-side timeouts.
- **Tuning tip**: Set to your target steady-state latency (for example, 80-120 ms).

#### Normal (Gaussian)

- **Behavior**: Bell curve centered on `mean_ms` with symmetric variation.
- **Parameters**: `mean_ms` (integer, > 0), `std_dev_ms` (integer, >= 0).
- **Use when**: Latency clusters around a typical value with moderate jitter.
- **Tuning tips**:
  - About 68% of samples fall within 1 standard deviation and 95% within 2.
  - Keep `std_dev_ms` smaller than `mean_ms` to avoid frequent clamping to 0.

#### Exponential

- **Behavior**: Many fast responses with a long tail of slower ones.
- **Parameters**: `mean_ms` (integer, > 0).
- **Use when**: You want realistic tail latency or occasional spikes.
- **Tuning tips**:
  - Approximate percentiles: p50 ~ `mean_ms * 0.69`, p95 ~ `mean_ms * 3`, p99 ~ `mean_ms * 4.6`.
  - Avoid very small `mean_ms` values (< 10) if you expect meaningful tail latency.

#### Uniform

- **Behavior**: Flat distribution between `min_ms` and `max_ms`.
- **Parameters**: `min_ms` (integer, >= 0), `max_ms` (integer, >= min_ms).
- **Use when**: You want bounded jitter without a central peak.
- **Tuning tip**: Set `min_ms` to your baseline latency and `max_ms` to your worst-case bound.

#### Choosing a Profile

- **Deterministic SLO checks**: Fixed
- **Stable services with jitter**: Normal
- **Tail-latency modeling**: Exponential
- **Hard bounds**: Uniform

#### Fixed

```json
{
  "type": "fixed",
  "value_ms": 100
}
```

#### Normal (Gaussian)

```json
{
  "type": "normal",
  "mean_ms": 100,
  "std_dev_ms": 20
}
```

#### Exponential

```json
{
  "type": "exponential",
  "mean_ms": 100
}
```

#### Uniform

```json
{
  "type": "uniform",
  "min_ms": 50,
  "max_ms": 150
}
```

### Response Object

```typescript
{
  status: 200,                           // HTTP status code
  body: "{\"users\": []}",              // JSON response body (as string)
  error_rate: 0.05                       // Probability of error (0.0–1.0)
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

None. Web Simulant supports unlimited concurrent requests (subject to system resources).

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
        "method": "GET",
        "path": "/api/test",
        "latency": {"type": "fixed", "value_ms": 100},
        "responses": [{"status": 200, "body": '{"ok": true}', "error_rate": 0.0}]
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
