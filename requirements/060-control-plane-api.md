# Control Plane API - Phase 1

## Overview

The Control Plane API runs on **port 8081** and provides management capabilities for the simulator. It handles configuration upload/download, validation, and serves the web UI.

**Purpose**:

- Upload and validate configurations
- Download current configuration
- Query active endpoints
- Serve web UI (static files)
- Monitor simulator status

**Port**: 8081  
**Engine Port**: 8080 (simulated endpoints)

---

## API Endpoints

### Configuration Management

#### POST /api/config/import

Upload and validate a configuration file. If valid, applies it to the simulator.

**Request**:

- Method: `POST`
- Content-Type: `multipart/form-data` OR `application/json` OR `application/x-yaml`
- Body: Configuration file (YAML or JSON)

**Multipart form example** (file upload from UI):

```http
POST /api/config/import HTTP/1.1
Host: localhost:8081
Content-Type: multipart/form-data; boundary=----WebKit

------WebKit
Content-Disposition: form-data; name="config"; filename="my-config.yaml"
Content-Type: application/x-yaml

version: "1.0"
...
------WebKit--
```

**JSON request example** (direct POST):

```http
POST /api/config/import HTTP/1.1
Host: localhost:8081
Content-Type: application/json

{
  "version": "1.0",
  "metadata": {...},
  "endpoints": [...]
}
```

**Success Response** (200 OK):

```json
{
  "status": "success",
  "message": "Configuration loaded successfully",
  "summary": {
    "endpoints_loaded": 5,
    "endpoints": [
      { "id": "get-users", "method": "GET", "path": "/api/users" },
      { "id": "create-user", "method": "POST", "path": "/api/users" }
    ]
  },
  "metadata": {
    "name": "user-api-basic",
    "description": "Basic user management API"
  }
}
```

**Validation Error Response** (400 Bad Request):

```json
{
  "status": "error",
  "message": "Configuration validation failed",
  "errors": [
    {
      "field": "endpoints[1].id",
      "error": "Duplicate endpoint ID 'get-users'",
      "location": "line 15"
    },
    {
      "field": "endpoints[2].latency.params.mean_ms",
      "error": "Value must be >= 0, got -10"
    }
  ],
  "previous_config_retained": true
}
```

**Parse Error Response** (400 Bad Request):

```json
{
  "status": "error",
  "message": "Failed to parse configuration file",
  "errors": [
    {
      "error": "Invalid YAML: expected a mapping at line 5, column 3"
    }
  ],
  "previous_config_retained": true
}
```

---

#### GET /api/config/export

Download the current active configuration.

**Request**:

- Method: `GET`
- Query parameters:
  - `format`: `yaml` (default) or `json`

**Example**:

```http
GET /api/config/export?format=yaml HTTP/1.1
Host: localhost:8081
```

**Success Response** (200 OK):

- Content-Type: `application/x-yaml` or `application/json`
- Content-Disposition: `attachment; filename="simulation-config-2026-02-11.yaml"`
- Body: Complete configuration file

```yaml
version: "1.0"
metadata:
  name: "user-api-basic"
  description: "Basic user management API"
  author: "web-simulant-team"
  created: "2026-02-11T10:00:00Z"
endpoints:
  - id: "get-users"
    method: "GET"
    path: "/api/users"
    ...
```

**No Configuration Response** (404 Not Found):

```json
{
  "status": "error",
  "message": "No configuration currently loaded"
}
```

---

#### POST /api/config/validate

Validate a configuration without applying it. Useful for checking syntax before upload.

**Request**:

- Method: `POST`
- Content-Type: `application/json` OR `application/x-yaml`
- Body: Configuration to validate

**Success Response** (200 OK):

```json
{
  "status": "valid",
  "message": "Configuration is valid",
  "summary": {
    "endpoints_count": 5,
    "endpoints": [{ "id": "get-users", "method": "GET", "path": "/api/users" }]
  },
  "warnings": [
    {
      "field": "endpoints[0].latency.params.mean_ms",
      "warning": "Very low latency (5ms) may show higher overhead percentage"
    }
  ]
}
```

**Validation Error Response** (400 Bad Request):

```json
{
  "status": "invalid",
  "message": "Configuration validation failed",
  "errors": [
    {
      "field": "endpoints[0].method",
      "error": "Invalid HTTP method 'GETS'. Must be GET, POST, PUT, DELETE, or PATCH"
    }
  ]
}
```

---

### Endpoint Management

#### GET /api/endpoints

List all currently active endpoints from loaded configuration.

**Request**:

- Method: `GET`

**Success Response** (200 OK):

```json
{
  "status": "success",
  "endpoints_count": 5,
  "endpoints": [
    {
      "id": "get-users",
      "method": "GET",
      "path": "/api/users",
      "latency": {
        "distribution": "normal",
        "params": { "mean_ms": 50, "stddev_ms": 10 }
      },
      "error_rate": 0.001,
      "response_status": 200
    },
    {
      "id": "create-user",
      "method": "POST",
      "path": "/api/users",
      "latency": {
        "distribution": "normal",
        "params": { "mean_ms": 150, "stddev_ms": 30 }
      },
      "error_rate": 0.02,
      "response_status": 201
    }
  ]
}
```

**No Configuration Response** (200 OK):

```json
{
  "status": "success",
  "endpoints_count": 0,
  "endpoints": [],
  "message": "No configuration loaded"
}
```

---

#### GET /api/endpoints/:id

Get details for a specific endpoint.

**Request**:

- Method: `GET`
- URL parameter: `id` - endpoint ID

**Example**:

```http
GET /api/endpoints/get-users HTTP/1.1
Host: localhost:8081
```

**Success Response** (200 OK):

```json
{
  "status": "success",
  "endpoint": {
    "id": "get-users",
    "method": "GET",
    "path": "/api/users",
    "latency": {
      "distribution": "normal",
      "params": { "mean_ms": 50, "stddev_ms": 10 }
    },
    "response": {
      "status": 200,
      "headers": {
        "Content-Type": "application/json"
      },
      "body": "{\"users\": [...]}"
    },
    "error_profile": {
      "rate": 0.001,
      "codes": [500],
      "body": "{\"error\": \"...\"}"
    }
  }
}
```

**Not Found Response** (404 Not Found):

```json
{
  "status": "error",
  "message": "Endpoint 'get-users-xyz' not found"
}
```

---

### Status & Health

#### GET /api/health

Control plane health check.

**Request**:

- Method: `GET`

**Response** (200 OK):

```json
{
  "status": "healthy",
  "control_plane": {
    "port": 8081,
    "uptime_seconds": 3661
  },
  "engine": {
    "port": 8080,
    "uptime_seconds": 3661,
    "configuration_loaded": true,
    "endpoints_active": 5
  }
}
```

---

#### GET /api/status

Detailed simulator status.

**Request**:

- Method: `GET`

**Response** (200 OK):

```json
{
  "status": "running",
  "uptime_seconds": 3661,
  "configuration": {
    "loaded": true,
    "name": "user-api-basic",
    "endpoints_count": 5,
    "loaded_at": "2026-02-11T10:00:00Z"
  },
  "engine": {
    "port": 8080,
    "listening": true
  },
  "control_plane": {
    "port": 8081,
    "listening": true
  },
  "version": "1.0.0",
  "build_info": {
    "rust_version": "1.75.0",
    "build_date": "2026-02-11"
  }
}
```

---

### Web UI

#### GET /

Serve the main web UI page.

**Request**:

- Method: `GET`

**Response** (200 OK):

- Content-Type: `text/html`
- Body: HTML page with embedded CSS/JS (or references to static assets)

**Features** (implemented in HTML/JS):

- Upload configuration file button
- Validate without applying button
- Download current configuration button
- Display list of active endpoints
- Display validation errors
- Show current configuration metadata
- Link to engine endpoints (port 8080)

---

#### GET /static/\*

Serve static assets (CSS, JS, images) for the web UI.

**Request**:

- Method: `GET`
- Path: `/static/<filename>`

**Examples**:

- `/static/style.css`
- `/static/app.js`
- `/static/favicon.ico`

**Response** (200 OK):

- Content-Type: Appropriate MIME type
- Body: File contents

**Not Found** (404 Not Found):

```json
{
  "status": "error",
  "message": "File not found"
}
```

---

## Error Response Format

All API errors follow a consistent format:

```json
{
  "status": "error",
  "message": "Human-readable error message",
  "errors": [
    {
      "field": "specific.field.path",
      "error": "Detailed error description",
      "location": "line 15" // optional
    }
  ],
  "request_id": "req-abc123" // optional, for debugging
}
```

---

## HTTP Status Codes

| Code | Usage                                                             |
| ---- | ----------------------------------------------------------------- |
| 200  | Success - request completed successfully                          |
| 201  | Created - resource created (not used in Phase 1)                  |
| 204  | No Content - success with no body (not used in Phase 1)           |
| 400  | Bad Request - validation error, parse error, invalid input        |
| 404  | Not Found - endpoint not found, config not loaded, file not found |
| 500  | Internal Server Error - unexpected error in control plane         |
| 503  | Service Unavailable - engine not ready (not used in Phase 1)      |

---

## Configuration Validation Details

### Validation Steps (in order)

1. **Parse**: Parse YAML/JSON syntax
2. **Schema**: Validate against schema structure
3. **Required Fields**: Check all required fields present
4. **Types**: Verify field types (string, number, etc.)
5. **Constraints**: Check ranges, uniqueness, valid enums
6. **Distribution Params**: Validate params match distribution type
7. **Logical**: Check for contradictions

### Common Validation Errors

**Duplicate Endpoint ID**:

```json
{
  "field": "endpoints[3].id",
  "error": "Duplicate endpoint ID 'get-users'. IDs must be unique.",
  "location": "line 42"
}
```

**Invalid Distribution Params**:

```json
{
  "field": "endpoints[0].latency.params.stddev_ms",
  "error": "Standard deviation must be > 0, got 0"
}
```

**Invalid Error Rate**:

```json
{
  "field": "endpoints[1].error_profile.rate",
  "error": "Error rate must be between 0.0 and 1.0, got 1.5"
}
```

**Missing Required Field**:

```json
{
  "field": "endpoints[2].method",
  "error": "Required field 'method' is missing"
}
```

**Invalid Path**:

```json
{
  "field": "endpoints[0].path",
  "error": "Path must start with '/', got 'api/users'"
}
```

**Duplicate Method+Path**:

```json
{
  "field": "endpoints[4]",
  "error": "Duplicate endpoint: GET /api/users already defined by endpoint 'get-users'"
}
```

---

## Validation Warnings (Non-blocking)

Warnings are informational - configuration is still accepted.

**Very Low Latency**:

```json
{
  "field": "endpoints[0].latency.params.delay_ms",
  "warning": "Very low latency (5ms) may show higher overhead percentage in test results"
}
```

**High Error Rate**:

```json
{
  "field": "endpoints[2].error_profile.rate",
  "warning": "High error rate (25%) - ensure this is intentional for resilience testing"
}
```

**Very Slow Latency**:

```json
{
  "field": "endpoints[3].latency.params.mean_ms",
  "warning": "Very high latency (10000ms) - requests may timeout"
}
```

---

## Content Type Handling

### Accepted Input Formats

**For configuration upload** (`/api/config/import`, `/api/config/validate`):

- `multipart/form-data` - File upload from web UI
- `application/json` - Direct JSON POST
- `application/x-yaml` - Direct YAML POST
- `text/yaml` - Alternative YAML content type
- `text/plain` - Will attempt to parse as YAML/JSON

**Auto-detection**:

- If content starts with `{`, parse as JSON
- Otherwise, parse as YAML

### Output Formats

**Configuration export** (`/api/config/export`):

- `application/x-yaml` - Default
- `application/json` - If `?format=json`

**API responses**:

- `application/json` - All API responses (status, errors, data)

**Web UI**:

- `text/html` - Main page
- `text/css` - Stylesheets
- `application/javascript` - Scripts

---

## CORS (Phase 2)

Phase 1: Control plane and UI served from same origin (port 8081), no CORS needed.

Phase 2: If external UI or cross-origin access needed, add CORS headers:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, OPTIONS
Access-Control-Allow-Headers: Content-Type
```

---

## Rate Limiting (Phase 2)

Phase 1: No rate limiting on control plane (single-user, local deployment).

Phase 2: Consider rate limiting if exposed externally:

- 60 requests/minute per IP for config operations
- Unlimited for health checks and status

---

## Authentication (Phase 2)

Phase 1: No authentication (localhost deployment, single-user).

Phase 2: If exposed externally, add:

- API key or basic auth
- Token-based auth for UI operations
- Read-only vs admin permissions

---

## Implementation Notes

### Configuration State Management

- Single active configuration at a time
- Configuration stored in memory for fast access
- Persisted to disk on successful import
- On startup, load last successful configuration
- On validation error, keep previous configuration active

### File Persistence

**Location**: `./config/active-config.yaml`

**On import success**:

1. Validate configuration
2. Apply to engine (create routes)
3. Write to disk (atomic write with temp file + rename)
4. Return success response

**On startup**:

1. Check if `./config/active-config.yaml` exists
2. If exists, load and validate
3. If valid, apply to engine
4. If invalid or missing, start with no configuration

### Response Times

Control plane operations should be fast:

- Health check: <5ms
- Status query: <10ms
- List endpoints: <20ms
- Validate config: <100ms (depends on complexity)
- Import config: <200ms (includes disk write)
- Export config: <50ms

### Logging

Log all control plane operations:

```
INFO  Configuration imported: user-api-basic (5 endpoints)
INFO  Configuration exported: format=yaml
WARN  Configuration validation failed: duplicate endpoint ID
ERROR Failed to write configuration to disk: permission denied
```

---

## Testing Considerations

### API Test Cases

1. **Import valid configuration**: 200 OK with summary
2. **Import invalid configuration**: 400 Bad Request with errors
3. **Import unparseable file**: 400 Bad Request with parse error
4. **Export configuration**: 200 OK with YAML/JSON content
5. **Export with no config loaded**: 404 Not Found
6. **Validate valid config**: 200 OK
7. **Validate invalid config**: 400 Bad Request
8. **List endpoints**: 200 OK with array
9. **Get specific endpoint**: 200 OK with details
10. **Get non-existent endpoint**: 404 Not Found
11. **Health check**: 200 OK always
12. **Status query**: 200 OK with details

### Integration Tests

1. **Upload config → List endpoints**: Verify endpoints appear
2. **Upload config → Export config**: Compare round-trip
3. **Upload config → Request endpoint on engine**: Verify behavior
4. **Upload invalid → Upload valid**: Verify recovery
5. **Restart simulator**: Verify config persisted

---

## Phase 2 Enhancements

- **Hot reload**: Change config without restart
- **Configuration history**: Keep last N configurations
- **Rollback**: Revert to previous configuration
- **Partial updates**: Update individual endpoints
- **Configuration templates**: Save/load templates
- **Import from URL**: Fetch config from external source
- **Metrics endpoint**: Request counts, latency stats per endpoint
- **WebSocket**: Real-time updates to UI
