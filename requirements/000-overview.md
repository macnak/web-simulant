# Project Overview

## 1. Brief Description

To produce a web simulation application. Docker image based system to simulate web application API behaviours.

This document is intended to eliminate assumptions, identify all gaps, and define decision criteria before any implementation starts. Any item marked as a gap is explicitly unknown and must be resolved.

## 2. Goals

The goals of this project are to cover the following areas:

- Provide a rich API set where behavior can be configured precisely and safely
- Produce predictable, repeatable measurements that match configured behavior
- Support common performance test styles (peak, capacity, spike, breakpoint, soak, endurance)
- Model queuing, saturation, and collapse in a controlled way
- Simulate realistic latency distributions with fine-grained tuning
- Generate large payloads efficiently without becoming the bottleneck
- Handle high concurrency while preserving deterministic behavior
- Allow fast, safe reconfiguration at run time with minimal downtime
- Support orchestration of behavior changes during a test
- Provide a web UI and admin APIs for configuration and operations
- Support workflow sequences and single endpoints with distinct performance profiles

Expected outcome for testers:

- The measured results of a test run should reflect the configured behavior patterns, so analysis tools and human reviewers can infer the simulated system conditions from the metrics and graphs.

Performance test styles and scenarios:

- Peak and capacity tests with sustained high load
- Spike tests with rapid load changes
- Breakpoint tests with deliberate saturation and failure patterns
- Soak or endurance tests with long-duration stability patterns
- Mixed-mode tests with scheduled changes in load and behavior

Behavior configuration dimensions:

- Response time distributions (fixed, Gaussian, log-normal, exponential, multi-modal)
- Distribution shape controls (mean, standard deviation, skewness, kurtosis, tail heaviness)
- Leading edge sharpness and peak width controls
- Time-window behavior changes (degradation, recovery, outages)
- Error injection (status codes, payload corruption, retry hints)
- Correlations (payload size vs latency, request type vs error rate)
- Queuing and backpressure effects (connection pool limits, concurrency caps)
- Throttling and rate limiting behaviors
- Network bandwidth constraints and transfer shaping
- Packet loss, retransmits, and jitter modeling
- Bursty network behavior and contention patterns
- Mobile network profiles (3G/4G/5G, high latency, variable throughput)
- DNS latency and resolution failures
- TLS handshake delays and connection setup overhead
- Connection retry behavior (client retries, server-side retry hints, idempotency considerations)
- Clustering and bursty error patterns
- Clustering of outlier response times and degraded performance windows
- Dynamic load-based behavior (performance degrades as concurrency rises)
- Long-run degradation patterns (buffers filling, memory leak-like latency growth)
- Batch-processing impact windows
- Workflow step dependencies and stateful sequences
- Load balancing behaviors (autoscale delays, session affinity issues, uneven node performance)
- Downstream dependency simulation (multi-hop calls, partial failure, error-in-payload with HTTP 200)
- Downstream outages with recovery windows and staged degradation
- Other anomaly types as explicitly configured

Future goals:

- OpenAPI file as the basis for API endpoints and performance profiles
- File upload and file download performance characteristics

## 3. Scope Boundaries (Explicit)

This section lists what is in-scope and out-of-scope. Each item must be resolved to avoid assumptions.

### 3.1 In-Scope (Gap)

- Early phase: local executable (no admin UI) to validate core engine behavior
- Initial non-admin APIs to simulate response times, distributions, and response codes
- Ability to model response time across network phases where feasible (e.g., connection setup vs response)
- Workflow-focused simulation as a first-class use case
- Moderate determinism: repeated runs match configured shapes with small variance

### 3.2 Out-of-Scope (Gap)

- Admin UI and full control plane in the initial local executable phase
- Multi-tenant hosting and SaaS features in early phases

### 3.3 Non-Goals (Gap)

- Replace production-grade load balancers or network appliances
- Provide performance testing tooling itself (focus is simulated system behavior)

## 4. Stakeholders and Users (Gap)

- Primary users: testers, performance engineers, statistical analysts, and engineers using performance tooling
- Secondary users (later phases): UI/UX testers validating workflow-style front ends (e.g., shopping, order, inventory flows)
- Tooling ecosystem: results should be consumable by JMeter, Playwright, Locust, k6, and similar tools
- Decision maker and final approver: project owner (user)
- Use of outputs: feed reporting tools and non-functional requirement checking tools
- Expected usage environments: research labs and performance testing environments, not CI/CD
- Preferred packaging: single local executable if the stack allows; otherwise a single Docker image bundling all required components
- Primary purpose of outputs: realistic, predefined behavior data for anomaly detection, NFR checking, and scoring validation

## 5. Use Cases and User Journeys

- Use cases (initial):
  - Protocol-based load tests (JMeter, k6, Locust) with predefined response characteristics
  - Workflow tests using simple linear API sequences
  - Anomaly detection dataset generation with controlled behavior patterns
  - NFR scoring validation using expected throughput and latency profiles
  - Long-run degradation detection (memory leak or buffer growth signatures)
- Success criteria for a valid run:
  - Configured latency distributions match within agreed tolerance
  - Error rate and clustering match configuration
  - Throughput/TPS remains within target range for the configured load profile
  - Phase-specific timing (DNS, TLS, connect) matches configured behavior where enabled
- User journey (early phase, API-first):
  - Define behavior configuration
  - Run external performance tool against simulator
  - Compare resulting measurements to configured profiles
- Outputs (early phase):
  - Minimal internal metrics; primary measurements derived from external tools and post-processing

## 6. Functional Requirements

List every capability as a testable requirement. Each item must be precise and unambiguous.

- The system shall expose API endpoints that return configurable responses (status code, headers, payload).
- The system shall support configurable latency distributions per endpoint or workflow step.
- The system shall support error injection patterns including clustered errors and error-in-payload responses.
- The system shall support time-window behavior changes (degradation, recovery, outages).
- The system shall support workflow definitions with simple linear sequences (early phase).
- The system shall allow shared API definitions reused across multiple workflows.
- The system shall support dynamic load-based behavior (performance degrades as concurrency rises).
- The system shall support network phase modeling where enabled (DNS, TLS, connect, first byte, transfer).
- The system shall support throttling, rate limiting, and connection retry behaviors.
- The system shall support downstream dependency simulation, including multi-hop failure propagation.
- The system shall support configuration reload without process restart (where feasible in the chosen stack).
- The system shall allow multiple scenarios to be defined and selected at run time.
- The system shall persist configurations with versioning metadata.
- The system shall provide a minimal health/status endpoint (early phase).
- The system shall provide a basic UI in the first release to add API endpoints and configure their behavior.
- The system shall support configuration import/export (download/upload) via the UI.
- The system shall provide admin UI and admin APIs in later phases (if control plane included).

## 7. Non-Functional Requirements

Define measurable thresholds for the following:

- Performance targets: max concurrency up to 10k overall; allow low per-endpoint concurrency (1 to 10) and very low TPS (sub-1 TPS) scenarios when configured. When no artificial delay is configured, latency overhead should reflect natural application and environment limits rather than a fixed target.
- Distribution accuracy: error tolerance for latency and error-rate distributions within 10% when sample size is at least 100; clarity improves around 1,000 samples.
- Determinism: probabilistic outputs; repeated runs need not match exactly, but large samples should converge to the configured shapes within tolerance.
- Guidance: document minimum sample sizes per distribution; at least 100 samples for basic tolerance, around 1,000 for clearer stability.
- Reliability: no crashes or data corruption during sustained tests of 8 to 12 hours; stability is a primary requirement to avoid wasted time from long reruns.
- Security: basic controls; admin access control if control plane enabled; no auth required for local-only single-user mode. Reduce attack surface and apply known security hardening where feasible.
- Operability: single executable or single Docker image deployment with minimal runtime dependencies.
- Resource efficiency: keep CPU and memory usage minimal while meeting performance targets.
- Usability: first-release UI should be simple enough for less-experienced users; advanced UI requirements apply only in later phases.
- Usability target: a new user can add a basic endpoint and behavior configuration in under 5 minutes without external documentation.

## 8. Data and Configuration Model

- Config schema shall include: scenarios, endpoints, workflows, distributions, error profiles, network profiles.
- Config format: YAML or JSON (human-editable, versionable).
- Endpoints should be uniquely identified (id + method + path) to prevent conflicting definitions.
- Grouping: workflows or endpoint groups may apply layered behavior overrides on top of base endpoint behavior.
- Conflict rules: duplicate method+path definitions are disallowed; shared APIs across workflows should reference a common base definition unless explicitly duplicated by design.
- Override precedence: group or workflow overrides must be explicit and documented (e.g., workflow overrides > group overrides > endpoint defaults).
- Validation rules shall reject invalid distributions or conflicting settings.
- Defaulting rules shall be explicit and documented for omitted fields.
- Configuration versioning should be supported; not required in the first release.

## 9. API Design

- Public API: simulation endpoints defined by configuration.
- Admin API (later phase): CRUD operations for scenarios, workflows, and distributions.
- Error model: map simulated error conditions to HTTP status and payload (including HTTP 200 with error payload).
- Rate limiting: behavior should be configurable per endpoint or workflow using TPS or TPH style limits; bandwidth limits apply to total bytes regardless of concurrency.
- Concurrency semantics: concurrency caps should apply per endpoint or group/workflow where defined; combined TPS can be below limits even with many sessions.

## 10. UI/UX Requirements

- UI flows (first release): add API endpoints and configure behavior for each endpoint.
- UI for importing/exporting configs (download/upload) in the first release.
- UI flows (later phase): create scenarios, edit behavior, save versions, select active scenario.
- UI for editing distributions and workflows (later phase).
- UI for OpenAPI import to seed endpoints and expected input/output behavior (later phase).

## 11. Observability and Diagnostics

- Logs: plain text with timestamps for start/stop and configuration changes; levels include DEBUG, INFO, WARN, ERROR. Allow verbosity to be reduced for normal use and avoid excessive log growth under load.
- Metrics: optional basic runtime counters (requests, errors) when enabled.
- Resource monitoring: CPU usage warning after 10 seconds above 80%, with elevated warnings at 90% and critical warning at 100% (first release). Connection limits of the host OS are a known practical constraint.
- Traceability: correlation IDs support if enabled by configuration.

## 12. Deployment and Operations

- Target deployment: local executable or single Docker image.
- Deployment scope (first release): single instance on a personal machine or a simple Docker container; no distributed or load-balanced deployment.
- Resource limits and scaling behavior: TBD based on engine benchmarks.
- Upgrade and migration strategy: TBD after config schema finalized.

## 13. Risks and Assumptions (Gap)

- Risks: potential conflicts between behavior definitions across shared API paths or workflows; may limit how much can be delivered in a single system without explicit override rules.
- Assumptions: proof-of-concept scope is acceptable for first release; additional risks will be identified as implementation progresses.

## 14. Research Topics (Gap)

Define research questions that must be answered before implementation. Each must include acceptance criteria for closure.

- Latency distribution fidelity at scale (accuracy vs throughput tradeoffs).
  - Acceptance: Target throughput achieved while keeping distribution error within agreed tolerance.
- Payload generation strategies without bottlenecks (static, templated, streamed).
  - Acceptance: Payload generation overhead under X% of response time budget.
- Deterministic simulation under high concurrency with moderate variance.
  - Acceptance: Repeated runs preserve distribution shapes and error patterns within agreed bounds.
- Runtime config hot-reload patterns and safety guarantees in Rust.
  - Acceptance: Config changes applied without process restart and without data race risks.
- Network phase modeling feasibility (DNS, TLS, connect, first byte, transfer).
  - Acceptance: Ability to attribute configured delays to specific phases in metrics.
- Workflow modeling approach (stateful vs stateless, shared APIs across workflows).
  - Acceptance: Workflow definitions are composable and testable with clear semantics.
- Error-in-payload modeling conventions (HTTP 200 with error payloads).
  - Acceptance: Schema supports explicit error payloads and downstream failure metadata.
- Load-balancing anomaly simulation (autoscale delays, session affinity drift).
  - Acceptance: Configurable patterns reproduce expected metric signatures.

## 15. Technology Stack Evaluation (Gap)

This section defines how technology choices will be evaluated and scored.

### 15.1 Candidates (Initial)

- Engine: Rust (current preference)
- Control plane: Node / TypeScript (current preference)
- Alternatives (engine): Go, Java, C++ (justify inclusion based on performance, safety, ecosystem).
- Alternatives (control plane): Go (HTMX/templating), Rust (Leptos/Actix), Python (FastAPI).

### 15.2 Evaluation Criteria (Gap)

- Performance and predictability under high concurrency
- Safety and correctness (memory safety, data race prevention)
- Ecosystem maturity for HTTP, metrics, config hot-reload
- Operability (Docker, metrics, logging, profiling support)
- Development velocity and maintainability
- Hiring availability and team familiarity
- Community and long-term support
- Integration with control plane and UI stack

### 15.3 Scoring Matrix (Gap)

Provide a weighted scoring table once criteria and weights are agreed.

- Scoring scale: 1 (poor) to 5 (excellent)
- Gap: Define weights for each criterion and target minimum thresholds.
- Gap: Apply scoring with evidence and references.

## 16. Project Scoring (Gap)

Provide an explicit scoring model for the project idea, goal, and objectives.

- Gap: Define scoring dimensions and thresholds (e.g., feasibility, ROI, risk, novelty, alignment).
- Gap: Apply scores once dimensions are agreed.

## 17. Open Questions (Gap)

- Gap: Consolidate unresolved questions here with owners and due dates.

## 18. Current Technology Stacks

### 18.1 Engine

This is where the core work for the simulation happens

- Rust

### 18.2 Control Plane

A simple Administration UI layer that allows configuration of the engine. There would be a simple authentication system and then access to the configuration options that can then be saved.

- Node / TypeScript
