# Configuration Schema - Phase 1

## Overview

The configuration schema defines endpoints, their behaviors, and how they respond to requests. This is the core data model for the system.

**Workflow**:

1. User creates/edits configuration (via UI or manually)
2. Upload configuration file (YAML/JSON)
3. System validates schema
4. If valid, server starts/reloads with new configuration
5. User can download current configuration for reuse/sharing

**Use cases**:

- Build libraries of common API patterns
- Share endpoint configurations across teams
- Version control configurations
- Quick setup for different testing scenarios

---

## Schema Format

Supports both YAML and JSON. Examples shown in YAML for readability.

### Top-Level Structure

```yaml
version: "1.0"
metadata:
  name: "user-api-simulation"
  description: "Simulates user management API endpoints"
  author: "test-team"
  created: "2026-02-11T10:00:00Z"

endpoints:
  - id: "get-users"
    # endpoint definition

  - id: "create-user"
    # endpoint definition

workflows: [] # Phase 2 feature, empty for now
```

---

## Endpoint Definition

Each endpoint simulates a specific API route with configurable behavior.

```yaml
- id: "get-users" # Unique identifier
  method: "GET" # HTTP method: GET, POST, PUT, DELETE, PATCH
  path: "/api/users" # Request path (exact match)

  latency:
    distribution: "normal" # Distribution type
    params:
      mean_ms: 100 # Mean latency in milliseconds
      stddev_ms: 20 # Standard deviation

  response:
    status: 200 # Success status code
    headers:
      Content-Type: "application/json"
    body: |
      {
        "users": [
          {"id": 1, "name": "Alice"},
          {"id": 2, "name": "Bob"}
        ]
      }

  error_profile:
    rate: 0.01 # 1% error rate
    codes: [500, 503] # Status codes to return on error
    body: | # Response body sent to client on error
      {
        "error": "Internal server error"
      }
```

---

## Request Matching (Optional)

By default, endpoints match based on `method` and `path` only. Request body matching is optional.

### Match Any Request (Default)

Omit the `request` section entirely, or use `body_match: "any"`:

```yaml
- id: "get-users"
  method: "GET"
  path: "/api/users"
  # No request section = matches any request
  latency:
    distribution: "fixed"
    params:
      delay_ms: 50
  response:
    status: 200
    body: '{"users": []}'
```

### Ignore Request Body

Explicitly ignore request body (same as omitting `request`):

```yaml
request:
  body_match: "ignore" # or "any"
```

### Match Exact Request Body

For POST/PUT endpoints where you want to validate exact request:

```yaml
- id: "create-user"
  method: "POST"
  path: "/api/users"
  request:
    body_match: "exact"
    body: |
      {
        "name": "Alice",
        "email": "alice@example.com"
      }
  response:
    status: 201
    body: '{"id": 123, "name": "Alice"}'
```

### Match Request Body Contains

Check if request body contains a substring:

```yaml
- id: "search-users"
  method: "POST"
  path: "/api/users/search"
  request:
    body_match: "contains"
    body: '"query"' # Matches any request with "query" in body
  response:
    status: 200
    body: '{"results": []}'
```

**Phase 1 Notes**:

- Request matching is simple and optional
- Focus remains on response behaviors (latency, errors, response body)
- UI will need input for both request and response bodies
- More sophisticated matching (JSON path, regex) deferred to Phase 2

---

## Latency Distributions

### Fixed Latency

Constant delay for all requests.

```yaml
latency:
  distribution: "fixed"
  params:
    delay_ms: 50
```

### Normal (Gaussian) Distribution

Most common pattern - latencies cluster around a mean.

```yaml
latency:
  distribution: "normal"
  params:
    mean_ms: 100
    stddev_ms: 20
```

### Exponential Distribution

Models random independent events (e.g., cache hits/misses).

```yaml
latency:
  distribution: "exponential"
  params:
    rate: 0.02 # 1/mean (mean = 50ms)
```

### Uniform Distribution

Random latency within a range.

```yaml
latency:
  distribution: "uniform"
  params:
    min_ms: 50
    max_ms: 150
```

---

## Error Profiles

Define how and when errors occur.

### Basic Error Profile

```yaml
error_profile:
  rate: 0.05 # 5% of requests fail
  codes: [500, 503] # Randomly select from these codes
  body: '{"error": "Service unavailable"}' # Response body on error
```

### No Errors

Omit `error_profile` or set rate to 0:

```yaml
error_profile:
  rate: 0
```

### Clustered Errors (Phase 2)

For future enhancement - errors occur in bursts:

```yaml
error_profile:
  rate: 0.05
  clustering:
    enabled: true
    window_ms: 5000
    burst_probability: 0.8
```

---

## Complete Example

```yaml
version: "1.0"
metadata:
  name: "e-commerce-api"
  description: "Simulates e-commerce API with realistic latencies"
  author: "performance-team"
  created: "2026-02-11T10:00:00Z"

endpoints:
  # Fast read endpoint (no request matching)
  - id: "get-products"
    method: "GET"
    path: "/api/products"
    # No request section = matches any GET to /api/products
    latency:
      distribution: "normal"
      params:
        mean_ms: 50
        stddev_ms: 10
    response:
      status: 200
      headers:
        Content-Type: "application/json"
      body: |
        {
          "products": [
            {"id": "p1", "name": "Widget", "price": 19.99},
            {"id": "p2", "name": "Gadget", "price": 29.99}
          ]
        }
    error_profile:
      rate: 0.001
      codes: [500]
      body: '{"error": "Database timeout"}'

  # Slower write endpoint (matches any request body)
  - id: "create-order"
    method: "POST"
    path: "/api/orders"
    request:
      body_match: "any" # Accept any request body
    latency:
      distribution: "normal"
      params:
        mean_ms: 200
        stddev_ms: 50
    response:
      status: 201
      headers:
        Content-Type: "application/json"
        Location: "/api/orders/12345"
      body: |
        {
          "order_id": "12345",
          "status": "pending"
        }
    error_profile:
      rate: 0.02
      codes: [500, 503]
      body: '{"error": "Order processing failed"}'

  # Fast cached endpoint
  - id: "get-user-profile"
    method: "GET"
    path: "/api/users/me"
    latency:
      distribution: "fixed"
      params:
        delay_ms: 10
    response:
      status: 200
      headers:
        Content-Type: "application/json"
        Cache-Control: "max-age=300"
      body: |
        {
          "user_id": "user123",
          "name": "Test User",
          "email": "test@example.com"
        }
    error_profile:
      rate: 0

workflows: []
```

---

## Validation Rules

### Required Fields

- `version`: Must be "1.0"
- `endpoints`: At least one endpoint required
- `endpoints[].id`: Unique across all endpoints
- `endpoints[].method`: Must be valid HTTP method
- `endpoints[].path`: Must start with `/`
- `endpoints[].latency.distribution`: Must be valid distribution type
- `endpoints[].latency.params`: Must match distribution requirements
- `endpoints[].response.status`: Valid HTTP status code
- `endpoints[].response.body`: Response body to return to client

### Optional Fields

- `endpoints[].request`: Request matching configuration (defaults to "any")
- `endpoints[].request.body_match`: "any", "exact", "contains", "ignore" (defaults to "any")
- `endpoints[].request.body`: Required if `body_match` is "exact" or "contains"

### Constraints

- **Unique endpoint identity**: No duplicate `(method, path)` combinations
- **Positive latencies**: All delay/mean/min/max values must be ≥ 0
- **Valid error rates**: `error_profile.rate` must be 0.0 to 1.0
- **Valid status codes**: Must be valid HTTP status codes (100-599)
- **Request body matching**: If `body_match` is "exact" or "contains", `body` must be provided
- **Distribution params**:
  - `normal`: `mean_ms ≥ 0`, `stddev_ms > 0`
  - `exponential`: `rate > 0`
  - `uniform`: `min_ms ≥ 0`, `max_ms > min_ms`
  - `fixed`: `delay_ms ≥ 0`

### Other Optional Fields

- `metadata`: All fields optional but recommended
- `error_profile`: Defaults to no errors if omitted
- `response.headers`: Defaults to `Content-Type: application/json` if omitted
- `workflows`: Empty list for Phase 1

---

## Validation Process

When configuration is uploaded:

1. **Schema validation**: Check structure matches expected format
2. **Field validation**: Verify all required fields present and correct types
3. **Constraint validation**: Check uniqueness, ranges, valid values
4. **Distribution validation**: Ensure params match distribution type
5. **Logical validation**: Check for contradictions or impossible configurations

**On validation success**:

- Configuration is accepted
- Server loads/reloads with new configuration
- UI shows success message with summary

**On validation failure**:

- Configuration is rejected
- UI shows specific validation errors with line numbers (if available)
- Server continues with previous configuration (if any)

---

## File Format Examples

### YAML (Recommended)

```yaml
version: "1.0"
metadata:
  name: "simple-api"
endpoints:
  - id: "health-check"
    method: "GET"
    path: "/health"
    # No request section = matches any request
    latency:
      distribution: "fixed"
      params:
        delay_ms: 5
    response:
      status: 200
      body: '{"status": "ok"}' # Response body sent to client
    error_profile:
      rate: 0
```

### JSON (Alternative)

```json
{
  "version": "1.0",
  "metadata": {
    "name": "simple-api"
  },
  "endpoints": [
    {
      "id": "health-check",
      "method": "GET",
      "path": "/health",
      "latency": {
        "distribution": "fixed",
        "params": {
          "delay_ms": 5
        }
      },
      "response": {
        "status": 200,
        "body": "{\"status\": \"ok\"}"
      },
      "error_profile": {
        "rate": 0
      }
    }
  ],
  "workflows": []
}
```

---

## Import/Export Workflow

### Upload Configuration (UI)

1. User clicks "Import Configuration"
2. File picker dialog (accepts .yaml, .yml, .json)
3. File uploaded to control plane API: `POST /api/config/import`
4. Server validates configuration
5. If valid:
   - Server reloads with new configuration
   - UI shows success: "Configuration loaded: 5 endpoints"
   - UI refreshes endpoint list
6. If invalid:
   - UI shows errors: "Validation failed: duplicate endpoint ID 'get-users'"
   - Previous configuration remains active

### Download Configuration (UI)

1. User clicks "Export Configuration"
2. UI makes request: `GET /api/config/export?format=yaml` (or `format=json`)
3. Server returns current configuration
4. Browser downloads file: `simulation-config-2026-02-11.yaml`
5. User can save to library, share, or version control

### Direct File Editing

1. User downloads current configuration
2. Edits locally in text editor
3. Uploads modified configuration
4. System validates and applies if valid

---

## Configuration Library Use Cases

### Scenario 1: Reusable API Patterns

```
library/
  ├── fast-read-api.yaml          # Cached read endpoints
  ├── slow-write-api.yaml         # Database write endpoints
  ├── unreliable-external.yaml    # Third-party API with errors
  └── user-management.yaml        # Complete user CRUD
```

### Scenario 2: Load Testing Scenarios

```
scenarios/
  ├── baseline-performance.yaml   # No errors, normal latencies
  ├── high-error-rate.yaml        # 10% errors for resilience testing
  ├── slow-backend.yaml          # 2x normal latencies
  └── mixed-load.yaml            # Combination of patterns
```

### Scenario 3: Environment Simulation

```
environments/
  ├── production-like.yaml        # Matches prod latencies
  ├── staging-like.yaml          # Staging characteristics
  └── degraded-service.yaml      # Simulates service degradation
```

---

## Phase 2 Enhancements (Future)

- **Workflows**: Group endpoints with override capabilities
- **Path parameters**: `/api/users/{id}` with parameter extraction
- **Advanced request matching**: JSON path, regex, headers, query params
- **Response templates**: Dynamic responses based on request data
- **Time-based behavior**: Different behavior at different times
- **Gradual degradation**: Latency/error rates that change over time
- **Request validation**: JSON schema validation, custom validators

---

## Implementation Notes

- Store configuration in memory for fast access
- Persist to disk for restart recovery
- Support hot reload without server restart (Phase 2)
- Validate on every import, before applying
- Provide clear error messages with context
- Log all configuration changes for audit trail
