# Phase 1 Implementation Plan

## Overview

Break down Phase 1 into concrete, implementable tasks with clear dependencies and completion criteria.

**Goal**: Working web simulator with configuration upload/download, dynamic endpoint routing, latency distributions, error injection, and simple web UI.

**Timeline**: Estimated 10-15 days of focused development

---

## Implementation Strategy

### Approach

1. **Bottom-up with integration checkpoints**
   - Build core libraries first (config, distributions)
   - Add engine functionality
   - Add control plane
   - Add UI last
   - Test integration at each layer

2. **Incremental validation**
   - Unit tests for each component
   - Integration tests after each major piece
   - Manual testing of UI workflows

3. **Deliverable milestones**
   - M1: Config loading works end-to-end
   - M2: Engine serves configured endpoints
   - M3: Control plane APIs functional
   - M4: UI fully operational

---

## Component Dependency Graph

```
┌─────────────────┐
│  Configuration  │  ← Foundation (no dependencies)
│   (parsing,     │
│   validation)   │
└────────┬────────┘
         │
         ├──────────────────┐
         │                  │
         ▼                  ▼
┌─────────────────┐   ┌──────────────┐
│  Distributions  │   │ Engine Core  │
│  (latency gen)  │   │ (routing,    │
│                 │   │  request     │
└────────┬────────┘   │  handling)   │
         │            └──────┬───────┘
         │                   │
         └───────┬───────────┘
                 │
                 ▼
        ┌────────────────────┐
        │  Endpoint Handler  │
        │  (apply latency,   │
        │   error injection) │
        └──────────┬─────────┘
                   │
                   ▼
        ┌────────────────────┐
        │   Control Plane    │
        │  (API endpoints,   │
        │   file handling)   │
        └──────────┬─────────┘
                   │
                   ▼
        ┌────────────────────┐
        │     Web UI         │
        │  (HTML + JS)       │
        └────────────────────┘
```

---

## Task Breakdown

### Phase 1.1: Project Setup (Day 1, ~2 hours)

**Goal**: Create project structure with proper dependency management

**Tasks**:

1. Create new Rust project structure
2. Setup Cargo.toml with dependencies
3. Define module structure
4. Create placeholder files
5. Setup basic logging

**Dependencies**: None

**Done Criteria**:

- [ ] `cargo build` succeeds
- [ ] Basic module structure in place
- [ ] Dependencies properly declared
- [ ] Logging framework configured

**Estimated time**: 2 hours

---

### Phase 1.2: Configuration Schema (Day 1-2, ~6 hours)

**Goal**: Parse and validate YAML/JSON configurations

**Tasks**:

1. Define Rust structs matching schema (serde)
2. Implement YAML parser
3. Implement JSON parser
4. Implement validation logic
5. Add comprehensive error messages
6. Write unit tests

**Dependencies**: Phase 1.1 (project setup)

**Done Criteria**:

- [ ] Parse valid YAML configurations
- [ ] Parse valid JSON configurations
- [ ] Detect all validation errors defined in schema
- [ ] Return clear error messages with locations
- [ ] 20+ unit tests covering all edge cases
- [ ] Example configs parse correctly

**Key Files**:

- `src/config/mod.rs` - Public API
- `src/config/schema.rs` - Struct definitions
- `src/config/parser.rs` - YAML/JSON parsing
- `src/config/validator.rs` - Validation logic
- `src/config/error.rs` - Error types

**Test Coverage**:

- Valid configs (use examples/)
- Missing required fields
- Invalid distribution params
- Duplicate endpoint IDs
- Duplicate method+path
- Invalid error rates
- Parse errors

**Estimated time**: 6 hours

---

### Phase 1.3: Distribution Implementations (Day 2-3, ~6 hours)

**Goal**: Generate latency values according to distributions

**Tasks**:

1. Define Distribution trait
2. Implement Fixed distribution
3. Implement Normal distribution
4. Implement Exponential distribution
5. Implement Uniform distribution
6. Write comprehensive unit tests
7. Test with Phase 0 benchmark parameters

**Dependencies**: Phase 1.2 (config schema for params)

**Done Criteria**:

- [ ] Fixed: returns exact delay
- [ ] Normal: mean/stddev within 5% at 1000 samples
- [ ] Exponential: mean within 5% at 1000 samples
- [ ] Uniform: uniform distribution validated
- [ ] All tests pass
- [ ] Performance: <1μs per sample generation

**Key Files**:

- `src/distributions/mod.rs` - Public API + trait
- `src/distributions/fixed.rs`
- `src/distributions/normal.rs`
- `src/distributions/exponential.rs`
- `src/distributions/uniform.rs`

**Test Coverage**:

- Each distribution: statistical properties
- Edge cases (zero stddev, negative params rejected)
- Performance tests

**Estimated time**: 6 hours

---

### Phase 1.4: Engine Core - Dynamic Router (Day 3-4, ~8 hours)

**Goal**: Route requests to correct endpoint handler based on method+path

**Tasks**:

1. Design endpoint registry structure
2. Implement request matching (method + path)
3. Build endpoint from configuration
4. Register all endpoints on startup
5. Handle route not found
6. Write unit and integration tests

**Dependencies**: Phase 1.2 (config schema)

**Done Criteria**:

- [ ] Match exact paths correctly
- [ ] Method matching works (GET vs POST)
- [ ] Return 404 for unknown routes
- [ ] Handle all HTTP methods
- [ ] Thread-safe for concurrent access
- [ ] Integration test: load config, route requests

**Key Files**:

- `src/engine/mod.rs` - Public API
- `src/engine/router.rs` - Request routing
- `src/engine/registry.rs` - Endpoint storage

**Test Coverage**:

- Route matching (exact path)
- Method matching
- 404 handling
- Concurrent access

**Estimated time**: 8 hours

---

### Phase 1.5: Engine Core - Endpoint Handler (Day 4-5, ~8 hours)

**Goal**: Apply latency and error injection to requests

**Tasks**:

1. Implement latency application (sleep)
2. Implement error injection (random + deterministic)
3. Return configured response or error response
4. Add request matching logic (if configured)
5. Write comprehensive tests
6. Integration test with distributions

**Dependencies**:

- Phase 1.3 (distributions)
- Phase 1.4 (routing)

**Done Criteria**:

- [ ] Apply latency from distribution
- [ ] Error injection matches configured rate
- [ ] Return correct response body
- [ ] Return correct headers
- [ ] Request matching works (any/exact/contains)
- [ ] Integration test: end-to-end request handling

**Key Files**:

- `src/engine/handler.rs` - Request handling
- `src/engine/response.rs` - Response building

**Test Coverage**:

- Latency application
- Error injection rates (verify with 10k samples)
- Response body/headers
- Request matching

**Estimated time**: 8 hours

---

### Phase 1.6: Engine Server (Day 5-6, ~4 hours)

**Goal**: HTTP server on port 8080 serving configured endpoints

**Tasks**:

1. Setup Axum server on port 8080
2. Integrate router with Axum
3. Handle configuration reload
4. Add graceful shutdown
5. Integration tests

**Dependencies**: Phase 1.5 (handler complete)

**Done Criteria**:

- [ ] Server listens on port 8080
- [ ] Serves all configured endpoints
- [ ] Applies latency and errors correctly
- [ ] Returns configured responses
- [ ] Handles concurrent requests
- [ ] Integration test: load config, make requests

**Key Files**:

- `src/engine/server.rs` - HTTP server setup
- `src/main.rs` - Engine startup

**Test Coverage**:

- Server starts and listens
- Concurrent request handling
- Configuration application

**Estimated time**: 4 hours

---

### Phase 1.7: Control Plane - API Endpoints (Day 6-8, ~10 hours)

**Goal**: REST API for configuration management on port 8081

**Tasks**:

1. Setup Axum server on port 8081
2. Implement POST /api/config/import
3. Implement GET /api/config/export
4. Implement POST /api/config/validate
5. Implement GET /api/endpoints
6. Implement GET /api/endpoints/:id
7. Implement GET /api/health
8. Implement GET /api/status
9. Add file persistence logic
10. Write API tests

**Dependencies**:

- Phase 1.2 (config validation)
- Phase 1.6 (engine server)

**Done Criteria**:

- [ ] All API endpoints functional
- [ ] Multipart file upload works
- [ ] JSON/YAML POST works
- [ ] Validation errors returned correctly
- [ ] Config persisted to disk
- [ ] Export returns correct format
- [ ] Integration test: API workflows

**Key Files**:

- `src/control_plane/mod.rs` - Public API
- `src/control_plane/server.rs` - HTTP server
- `src/control_plane/handlers/` - API handlers
  - `config.rs` - Config upload/download/validate
  - `endpoints.rs` - Endpoint queries
  - `status.rs` - Status/health
- `src/control_plane/persistence.rs` - File I/O

**Test Coverage**:

- Each API endpoint
- Error handling
- File upload (multipart)
- Validation workflows
- Persistence

**Estimated time**: 10 hours

---

### Phase 1.8: Web UI - HTML (Day 8-9, ~4 hours)

**Goal**: Static HTML page with structure and styling

**Tasks**:

1. Create index.html with structure
2. Add embedded CSS
3. Design layout (header, panels, modals)
4. Add responsive styling
5. Test in browser

**Dependencies**: Phase 1.7 (control plane APIs)

**Done Criteria**:

- [ ] HTML structure matches wireframe
- [ ] CSS styling applied
- [ ] Proper semantic HTML
- [ ] Accessibility features (ARIA labels)
- [ ] Looks good in Chrome/Firefox

**Key Files**:

- `static/index.html` - Main UI page

**Test Coverage**:

- Manual browser testing
- Accessibility check

**Estimated time**: 4 hours

---

### Phase 1.9: Web UI - JavaScript (Day 9-10, ~6 hours)

**Goal**: Interactive functionality for UI

**Tasks**:

1. File upload logic
2. API request handling (fetch)
3. Success/error modal display
4. Endpoint list rendering
5. Download trigger
6. Refresh functionality
7. Expand/collapse endpoint details
8. Copy curl command

**Dependencies**: Phase 1.8 (HTML structure)

**Done Criteria**:

- [ ] Upload config works
- [ ] Validation errors displayed
- [ ] Download config works
- [ ] Endpoint list refreshes
- [ ] Expand/collapse works
- [ ] Copy button works
- [ ] All workflows functional

**Key Files**:

- `static/index.html` - Embedded JS or separate file
- `static/app.js` - If separated

**Test Coverage**:

- Manual testing of all workflows
- Browser compatibility (Chrome, Firefox)

**Estimated time**: 6 hours

---

### Phase 1.10: Integration & Testing (Day 10-12, ~8 hours)

**Goal**: End-to-end testing and bug fixes

**Tasks**:

1. Load example configs via UI
2. Test each endpoint on engine
3. Test all UI workflows
4. Load testing (stress test)
5. Fix bugs
6. Performance profiling
7. Write integration test suite
8. Documentation updates

**Dependencies**: All previous phases

**Done Criteria**:

- [ ] All 4 example configs upload successfully
- [ ] Engine serves endpoints correctly
- [ ] Latency and errors match configuration
- [ ] UI workflows complete without errors
- [ ] Load test: 10k concurrent, no crashes
- [ ] All integration tests pass
- [ ] Documentation up to date

**Test Coverage**:

- Upload each example config
- Make requests to each endpoint
- Verify latency distributions
- Verify error rates
- UI workflow testing

**Estimated time**: 8 hours

---

### Phase 1.11: Docker & Deployment (Day 12-13, ~6 hours)

**Goal**: Docker image that runs full simulator

**Tasks**:

1. Create Dockerfile
2. Multi-stage build
3. Config directory mounting
4. Port mapping (8080, 8081)
5. Test Docker build and run
6. Update documentation
7. Create docker-compose (optional)

**Dependencies**: Phase 1.10 (system fully functional)

**Done Criteria**:

- [ ] Docker image builds
- [ ] Image size reasonable (<100MB)
- [ ] Runs on Docker
- [ ] Ports accessible
- [ ] Config directory works
- [ ] Documentation updated

**Key Files**:

- `Dockerfile`
- `docker-compose.yml` (optional)
- `README.md` - Docker instructions

**Estimated time**: 6 hours

---

### Phase 1.12: Polish & Documentation (Day 13-15, ~8 hours)

**Goal**: Production-ready release

**Tasks**:

1. Clean up code
2. Add code comments
3. Complete README
4. Write user guide
5. Create video demo (optional)
6. Performance optimization
7. Final testing
8. Tag v1.0.0 release

**Dependencies**: Phase 1.11 (Docker complete)

**Done Criteria**:

- [ ] Code reviewed and cleaned
- [ ] README comprehensive
- [ ] User guide complete
- [ ] All tests passing
- [ ] Performance acceptable
- [ ] Release tagged

**Estimated time**: 8 hours

---

## Milestone Definitions

### M1: Configuration Loading (End of Day 2)

**Components**: Phase 1.1, 1.2

**Validation**:

```rust
cargo test --lib config
cargo run -- --validate examples/01-simple-health-check.yaml
```

**Exit Criteria**:

- All example configs parse correctly
- Validation catches all error types
- Error messages are clear

---

### M2: Engine Functional (End of Day 5)

**Components**: Phase 1.3, 1.4, 1.5, 1.6

**Validation**:

```bash
cargo run &  # Start engine
curl -X POST -H "Content-Type: application/yaml" \
  --data-binary @examples/01-simple-health-check.yaml \
  http://localhost:8080/__config

curl http://localhost:8080/health
# Should return configured response with ~5ms latency
```

**Exit Criteria**:

- Engine starts and listens on 8080
- Loads configuration
- Serves endpoints with correct latency
- Error injection works

---

### M3: Control Plane Functional (End of Day 8)

**Components**: Phase 1.7

**Validation**:

```bash
cargo run &  # Start both engine and control plane

# Upload config via API
curl -X POST -F "config=@examples/02-user-api-basic.yaml" \
  http://localhost:8081/api/config/import

# List endpoints
curl http://localhost:8081/api/endpoints

# Download config
curl http://localhost:8081/api/config/export?format=yaml
```

**Exit Criteria**:

- All API endpoints functional
- Config persists to disk
- Validation errors returned properly

---

### M4: UI Functional (End of Day 10)

**Components**: Phase 1.8, 1.9

**Validation**:

1. Open http://localhost:8081 in browser
2. Upload examples/03-ecommerce-mixed.yaml
3. Verify endpoints list displays
4. Download configuration
5. Test each workflow

**Exit Criteria**:

- All UI workflows work
- No JavaScript errors
- Validation errors display correctly

---

## Development Environment

### Required Tools

- Rust 1.75+ (stable)
- Cargo
- curl (for testing)
- Docker (for Phase 1.11)
- Browser (Chrome or Firefox)

### Recommended Setup

- VS Code with rust-analyzer
- Terminal with multiple tabs/panes
- Postman or similar (optional, for API testing)

---

## Testing Strategy

### Unit Tests

- Each module has unit tests
- Run: `cargo test --lib`
- Coverage goal: >80%

### Integration Tests

- End-to-end workflows
- Located in `tests/`
- Run: `cargo test --test integration`

### Manual Testing

- UI workflows
- Load testing
- Performance validation

### Continuous Testing

- Run tests after each phase
- Don't proceed if tests fail

---

## Risk Mitigation

### Risk: Latency distribution accuracy

**Mitigation**: Phase 1.3 includes statistical validation tests

**Contingency**: Compare with Phase 0 benchmark results

---

### Risk: Concurrent request handling

**Mitigation**: Load testing in Phase 1.10

**Contingency**: Add connection pooling if needed

---

### Risk: UI complexity grows

**Mitigation**: Keep it simple, no frameworks

**Contingency**: Can simplify UI if needed, core functionality is API

---

### Risk: Configuration reload complexity

**Mitigation**: Phase 1 doesn't require hot reload, restart is acceptable

**Contingency**: Defer hot reload to Phase 2

---

## Success Metrics

### Functional

- [ ] All 4 example configs upload successfully
- [ ] All API endpoints work
- [ ] Engine serves configured endpoints
- [ ] Latency within 10% of configuration
- [ ] Error rates within 1% of configuration
- [ ] UI workflows complete without errors

### Performance

- [ ] Config upload: <500ms
- [ ] Engine latency overhead: <2ms (validated in Phase 0)
- [ ] Control plane APIs: <100ms response
- [ ] UI page load: <200ms

### Quality

- [ ] All tests passing
- [ ] Code reviewed
- [ ] Documentation complete
- [ ] Docker image works

---

## Timeline Summary

| Phase     | Days            | Component           | Milestone        |
| --------- | --------------- | ------------------- | ---------------- |
| 1.1       | 0.25            | Project setup       | -                |
| 1.2       | 0.75            | Config schema       | M1 (Day 2)       |
| 1.3       | 0.75            | Distributions       | -                |
| 1.4       | 1.0             | Router              | -                |
| 1.5       | 1.0             | Handler             | -                |
| 1.6       | 0.5             | Engine server       | M2 (Day 5)       |
| 1.7       | 1.25            | Control plane API   | M3 (Day 8)       |
| 1.8       | 0.5             | UI HTML             | -                |
| 1.9       | 0.75            | UI JavaScript       | M4 (Day 10)      |
| 1.10      | 1.0             | Integration testing | -                |
| 1.11      | 0.75            | Docker              | -                |
| 1.12      | 1.0             | Polish & docs       | Release (Day 15) |
| **Total** | **~10-15 days** |                     |                  |

---

## Next Steps

1. Review this plan
2. Confirm approach and timeline
3. Begin Phase 1.1: Project Setup
4. Track progress in PROJECT-STATUS.md
5. Update documentation as completed

---

## Notes

- Timeline assumes focused development (6-8 hours/day)
- Adjust for actual pace after M1
- Can parallelize some tasks if multiple developers
- Buffer time included for debugging and iteration
- Performance already validated in Phase 0
