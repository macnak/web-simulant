# Configuration Examples

This directory contains example configurations demonstrating different use cases and patterns for the web simulator.

## Overview

Each example configuration is a complete, valid YAML file that can be uploaded directly to the simulator. These examples serve as:

- **Learning resources** - understand the configuration schema through concrete examples
- **Starting templates** - copy and modify for your own use cases
- **Test fixtures** - validate the implementation against known configurations
- **Library patterns** - reusable configurations for common scenarios

## Examples

### 01-simple-health-check.yaml

**Purpose**: Minimal configuration with a single endpoint

**Use case**: Testing basic simulator functionality, smoke tests

**Characteristics**:

- Single endpoint: `GET /health`
- Fixed latency: 5ms
- No errors
- Minimal response body

**Best for**: Quick validation, initial setup testing

---

### 02-user-api-basic.yaml

**Purpose**: Basic CRUD operations for a user management API

**Use case**: Standard REST API simulation with read and write operations

**Characteristics**:

- 5 endpoints: list, get, create, update, delete
- Mix of fixed and normal distributions
- Low error rates (0.1% - 3%)
- Realistic response bodies with user data
- Different latencies for reads (fast) vs writes (slower)

**Best for**: General-purpose API testing, learning multiple endpoint patterns

---

### 03-ecommerce-mixed.yaml

**Purpose**: E-commerce API with varied behaviors

**Use case**: Complex application testing with different performance profiles

**Characteristics**:

- 6 endpoints: products, search, cart, checkout, orders
- All four distribution types demonstrated
- Varied error rates (0.05% - 5%)
- Realistic latencies: fast reads (25-45ms) to slow operations (350ms)
- Different reliability profiles per endpoint

**Best for**: Load testing, performance profiling, realistic scenarios

**Key patterns**:

- Product catalog: fast, cached, very reliable
- Search: variable latency (exponential distribution)
- Cart operations: moderate speed and reliability
- Checkout: slowest, highest error rate (5%)
- Order status: fast, reliable

---

### 04-unreliable-external.yaml

**Purpose**: Simulate unreliable third-party services

**Use case**: Resilience testing, circuit breaker validation, retry logic testing

**Characteristics**:

- 5 endpoints simulating external services
- High error rates (8% - 25%)
- Variable and slow latencies (200ms - 5000ms)
- Realistic failure scenarios: rate limits, timeouts, gateway errors
- Mix of HTTP error codes: 429, 500, 502, 503, 504

**Best for**: Chaos engineering, resilience testing, failure scenario testing

**Key patterns**:

- Weather API: 10% errors, variable latency
- Payment gateway: 8% errors, critical path simulation
- Rate-limited API: 15% errors (mostly 429)
- Legacy system: 20% errors, very slow (500-2000ms)
- Slow reports: 25% errors, extreme latency (1-5 seconds)

---

## Usage

### Import Configuration

1. Start the web simulator
2. Navigate to the web UI (http://localhost:8081)
3. Click "Import Configuration"
4. Select one of these YAML files
5. System validates and loads the configuration
6. Endpoints are now available on port 8080

### Test Endpoints

Once loaded, test endpoints using curl:

```bash
# Example: test health check
curl http://localhost:8080/health

# Example: test user list
curl http://localhost:8080/api/users

# Example: test product catalog
curl http://localhost:8080/api/products
```

### Modify for Your Use Case

1. Download an example configuration
2. Edit in your favorite text editor
3. Modify endpoints, latencies, error rates
4. Upload modified configuration
5. Test and iterate

---

## Configuration Patterns

### Low-Error, High-Reliability (0.1% - 1% errors)

Use for: production-like scenarios, happy path testing

Examples: health checks, cached reads, product catalogs

```yaml
error_profile:
  rate: 0.001 # 0.1%
  codes: [500]
  body: '{"error": "Rare failure"}'
```

### Moderate Errors (2% - 5% errors)

Use for: write operations, realistic testing, moderate resilience

Examples: user creation, cart operations, checkouts

```yaml
error_profile:
  rate: 0.03 # 3%
  codes: [400, 500]
  body: '{"error": "Operation failed"}'
```

### High Errors (10% - 25% errors)

Use for: resilience testing, chaos engineering, failure scenarios

Examples: external APIs, legacy systems, rate-limited services

```yaml
error_profile:
  rate: 0.15 # 15%
  codes: [429, 500, 503]
  body: '{"error": "Service unavailable"}'
```

### Fast Operations (<50ms)

Use for: cached reads, health checks, simple queries

```yaml
latency:
  distribution: "fixed"
  params:
    delay_ms: 10
```

### Normal Operations (50-200ms)

Use for: typical API operations, database queries

```yaml
latency:
  distribution: "normal"
  params:
    mean_ms: 100
    stddev_ms: 20
```

### Slow Operations (200ms+)

Use for: complex queries, external APIs, batch operations

```yaml
latency:
  distribution: "uniform"
  params:
    min_ms: 500
    max_ms: 2000
```

---

## Next Steps

1. **Try each example** - Load and test each configuration
2. **Mix and match** - Combine endpoints from different examples
3. **Create your own** - Build configurations for your specific use cases
4. **Share configurations** - Build a library of reusable patterns
5. **Version control** - Store configurations in git for team sharing

---

## Validation

All examples in this directory are valid against the schema defined in:
`requirements/050-configuration-schema.md`

If an example fails validation after a schema change, it should be updated to maintain compatibility.
