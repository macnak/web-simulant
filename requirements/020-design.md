# Design

This document will describe the system architecture, component boundaries, and design decisions.

## 1. Architecture Overview

High-level structure:

- Engine: core simulation runtime
- Control plane: configuration UI and admin APIs
- Configuration store: persisted endpoint/workflow definitions

Deployment model: single executable or single Docker image.

## 2. Components

### Engine (Phase 1)

- HTTP server: accepts incoming requests matching configured endpoints
- Behavior engine: applies configured latency distributions, error injection, network profiles
- Request router: matches incoming requests to endpoint configurations
- Distribution sampler: generates delays from Normal, Exponential, Uniform, Fixed distributions
- Metrics collector: tracks request counts, latencies, error rates

### Control Plane (Phase 1)

- Web UI: simple form-based interface to define/edit endpoints and workflows
- Configuration API: REST endpoints for CRUD operations on endpoint/workflow definitions
- Configuration store: file-based persistence (YAML/JSON) with in-memory cache
- Admin endpoints: health check, metrics export, configuration reload

### Phase 1 Scope

- Single process combining engine and control plane
- No authentication/authorization (localhost binding recommended)
- File-based configuration persistence
- Manual Docker image build (no auto-scaling or orchestration)

### API Surface

**Engine endpoints** (port 8080):

- User-configured simulation endpoints matching defined paths/methods
- Behavior shaped by endpoint configuration (latency, errors, network profiles)

**Control plane endpoints** (port 8081):

- GET / - Web UI dashboard
- GET /admin/health - Health check
- GET /admin/metrics - Metrics export (JSON)
- POST /api/endpoints - CRUD for endpoint configurations
- POST /api/workflows - CRUD for workflow configurations
- POST /api/config/import - Import YAML/JSON config
- GET /api/config/export - Export current config

## 3. Data Flow

### Request Flow (Engine)

1. HTTP request arrives at engine listener
2. Router matches request to endpoint configuration by method + path
3. If no match, return 404
4. If matched, apply workflow overrides (if request is part of active workflow)
5. Sample latency from configured distribution
6. Apply delay (async sleep)
7. Check error injection rules; decide success vs error
8. If error, return configured error response (status code, body)
9. If success, return configured success response
10. Record metrics (latency, status, endpoint id)

### Configuration Flow (Control Plane)

1. User submits endpoint/workflow via UI form or API
2. Validate schema (required fields, valid distributions, no conflicts)
3. Persist to configuration file
4. Update in-memory cache
5. Log configuration change
6. Return success/error to UI

## 4. Configuration Loading and Overrides

### Loading Priority

1. Read configuration file at startup (YAML/JSON)
2. Validate all endpoint and workflow definitions
3. Build in-memory index by endpoint identity (method + path)
4. Reject duplicate endpoint definitions
5. Fail startup if configuration is invalid

### Override Rules (Phase 1)

- Workflow configuration overrides endpoint defaults for matching requests
- Precedence: workflow > endpoint
- No group-level overrides in Phase 1
- Conflicts: reject at validation time (e.g., same endpoint in multiple workflows)

### Reload Behavior (Phase 1)

- Configuration changes via UI/API are persisted and applied immediately
- No hot reload of file from disk in Phase 1
- Manual restart required if editing configuration file directly

## 5. Error Handling and Resilience

### Error Handling Strategy

- Invalid configuration: fail at validation time, return clear error message
- Request timeout: respect client timeout, cancel async delay if client disconnects
- Out of memory: fail gracefully, log error, avoid panic
- Configuration corruption: detect at load time, refuse to start with corrupted config
- Panic recovery: use Rust panic handlers to log and attempt graceful shutdown

### Resilience Targets

- 8-12 hour soak without crash (Phase 1 validation requirement)
- CPU efficiency: avoid saturation under configured load (warn at >80% sustained)
- Memory stability: no unbounded growth, no leaks
- Distribution accuracy: degrade gracefully under CPU saturation, emit warnings

## 6. Observability

### Logging (Phase 1)

- Format: plain text with timestamps
- Levels: DEBUG, INFO, WARN, ERROR
- Reduced verbosity under load (log sampling for high-frequency events)
- Startup/shutdown: log configuration summary
- Request logging: optional per-request logging at DEBUG level
- Configuration changes: log all create/update/delete operations at INFO
- CPU warnings: log when >80% for 10s (WARN), >90% (ERROR), 100% (CRITICAL)

### Metrics Export (Phase 1)

- Admin endpoint: GET /admin/metrics (JSON format)
- Per-endpoint counters: request count, success count, error count
- Per-endpoint histograms: p50/p95/p99 latency
- Global: total requests, current concurrency, uptime
- No time-series database in Phase 1; metrics are point-in-time snapshots

## 7. Deployment Model

### Phase 1 Deployment

- Single Docker image containing engine + control plane
- Single process, multi-threaded (Tokio async runtime for Rust)
- Volume mount for configuration persistence (optional)
- Environment variables for runtime config (log level, bind address, ports)
- No load balancer or orchestration; single instance only

### Port Allocation

- Engine listener: default 8080 (configurable via env)
- Control plane UI/API: default 8081 (configurable via env)
- Health check: GET /admin/health on control plane port

### Docker Build

- Multi-stage build: Rust toolchain -> static binary -> minimal runtime image (distroless or alpine)
- Binary size target: <50 MB for Phase 1
- Startup time target: <2 seconds

## 8. Design Risks and Tradeoffs

### 8.1 Single-Stack vs Mixed-Stack

Single-stack (Rust for both engine and control plane):

- Pros: single runtime, simpler build/deploy, max performance headroom, better long-term stability.
- Cons: slower UI iteration velocity compared to Node/Go.

Mixed-stack (Rust engine + Go/Node control plane):

- Pros: faster UI iteration, broader control-plane ecosystem.
- Cons: two runtimes, more build complexity, inter-process boundaries.

Decision rationale:

- Engine performance is the top priority (weight 45).
- UI is simple and iteration speed is secondary.
- Industry trend shows Rust replacing Node/TypeScript for stability and performance, including UI systems.
- Recommendation: single-stack Rust for both engine and control plane to maximize performance headroom and simplify deployment.
