# Phase 2 Implementation Plan

## Overview

Phase 2 expands behavior modeling and control plane capabilities beyond the v1.0.0 scope.

**Goal**: Add advanced behavior modes (distributions, time windows, error modes, rate limits), CRUD APIs with UI editing, and basic observability/versioning.

**Estimated timeline**: 6-10 weeks (split into 3 sub-phases for incremental delivery)

---

## Implementation Strategy

### Approach

1. **Behavior-first**
   - Extend the core engine with new behaviors before UI and CRUD tooling.
2. **API-first for CRUD**
   - Add endpoints and validation before wiring the UI.
3. **Observable increments**
   - Each sub-phase ends with new examples and tests.

### Deliverable Milestones

- **M5**: New behavior modes validated (distributions, time windows, error modes, rate limits)
- **M6**: Endpoint CRUD + UI editor functional
- **M7**: Observability + versioning in place

---

## Component Dependency Graph

```
┌────────────────────┐
│  Config Schema v2  │  ← Add new behaviors, version metadata
└─────────┬──────────┘
          │
          ▼
┌────────────────────┐
│  Engine Behaviors  │  ← Distributions, time windows, rate limits
└─────────┬──────────┘
          │
          ▼
┌────────────────────┐
│  Control Plane API │  ← CRUD, metrics, versioning
└─────────┬──────────┘
          │
          ▼
┌────────────────────┐
│      Web UI        │  ← Editor, conflict reporting, status
└────────────────────┘
```

---

## Sub-Phase 2.1: Behavior Modeling Expansion (3-4 weeks)

**Goal**: Add behavior capabilities to the engine and config schema.

**Tasks**:

1. **Latency distributions**
   - Add log-normal distribution
   - Add multi-modal distribution (mixture)
2. **Time-window behavior changes**
   - Define schedule blocks in schema
   - Apply latency/error overrides by time window
3. **Error injection extensions**
   - Error-in-payload (HTTP 200 with error body)
   - Payload corruption (truncate or replace)
4. **Rate limiting and bandwidth caps**
   - Token bucket per endpoint/workflow
   - Optional bandwidth cap on response body
5. **Validation + examples**
   - Schema validation updates
   - Add 2-3 example configs
6. **Tests**
   - Statistical tests for new distributions
   - Time-window behavior tests
   - Rate limiting enforcement tests

**Dependencies**: Phase 1 complete (engine + config + control plane)

**Done Criteria**:

- [ ] Log-normal and multi-modal distributions pass 10% tolerance at 1000 samples
- [ ] Time-window behavior changes apply at correct times
- [ ] Error-in-payload and payload corruption work with clear config flags
- [ ] Rate limiting enforced under load
- [ ] New example configs validate and run
- [ ] Unit tests added for all new behaviors

**Key Files**:

- `src/distributions/log_normal.rs`
- `src/distributions/mixture.rs`
- `src/config/schema.rs`
- `src/config/validator.rs`
- `src/engine/handler.rs`
- `src/engine/response.rs`
- `examples/` (new Phase 2 examples)

**Estimated time**: 3-4 weeks

---

## Sub-Phase 2.2: Control Plane CRUD + UI Editor (2-3 weeks)

**Goal**: Add endpoint CRUD APIs and a UI editor for add/edit/delete.

**Tasks**:

1. **CRUD API endpoints**
   - Create/update/delete endpoint definitions
   - Validate and persist changes
2. **Conflict detection**
   - Handle method+path collisions
   - Display conflicts in API and UI
3. **UI editor**
   - Form-based endpoint editor
   - Live validation feedback
   - Save and apply changes
4. **Schema compatibility**
   - Ensure CRUD updates align with v2 schema
5. **Tests**
   - API integration tests
   - UI flow sanity checks

**Dependencies**: Sub-phase 2.1 (schema updates)

**Done Criteria**:

- [ ] CRUD endpoints in control plane work end-to-end
- [ ] UI can create, edit, and delete endpoints
- [ ] Validation errors are surfaced clearly
- [ ] Conflicts are detected and reported
- [ ] Config persistence works with CRUD edits

**Key Files**:

- `src/control_plane/handlers.rs`
- `src/control_plane/persistence.rs`
- `static/index.html` (editor UI)
- `static/` JS for form handling

**Estimated time**: 2-3 weeks

---

## Sub-Phase 2.3: Observability + Versioning (1-2 weeks)

**Goal**: Add config version metadata, metrics, and CPU warnings.

**Tasks**:

1. **Config version metadata**
   - Add version id, author, timestamp
   - Persist and export metadata
2. **Metrics endpoint**
   - Basic counters: requests, errors, latency
   - Per-endpoint stats (optional toggle)
3. **CPU warning thresholds**
   - Log warnings at >80%, >90%, >100%
4. **Docs and examples**
   - Document new fields and endpoints

**Dependencies**: Sub-phase 2.2 (CRUD + persistence)

**Done Criteria**:

- [ ] Config exports include version metadata
- [ ] Metrics endpoint provides basic counters
- [ ] CPU warnings appear when thresholds are exceeded
- [ ] Documentation updated with new fields

**Key Files**:

- `src/control_plane/handlers.rs`
- `src/engine/metrics.rs` (new)
- `src/config/schema.rs`
- `API.md` (doc updates)

**Estimated time**: 1-2 weeks

---

## Test Strategy

- **Unit tests** for new distributions and validation rules
- **Integration tests** for time-window behaviors and rate limiting
- **Control plane tests** for CRUD and persistence
- **Smoke tests** with new example configs

---

## Phase 2 Risks and Mitigations

- **Complexity creep**: Keep schema changes backward compatible where possible
- **Performance overhead**: Benchmark distribution sampling and rate limits
- **UI scope**: Start with endpoint CRUD only; leave workflows for Phase 3

---

## Proposed Phase 2 Deliverables

- Updated config schema with new behavior blocks
- New distribution implementations (log-normal, mixture)
- Time-window behavior scheduling
- Error-in-payload and payload corruption options
- Rate limiting and bandwidth caps
- Endpoint CRUD APIs + UI editor
- Metrics endpoint + CPU warnings
- Versioned config metadata
- New example configs and tests
