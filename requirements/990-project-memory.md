# Project Memory

Purpose: keep a concise record of decisions and constraints so future LLM sessions remain focused on this project.

## How to Use

- Read this file first in a new session.
- Keep entries concise and factual; avoid speculation.
- Update after decisions change or new constraints are added.
- Link to source documents when relevant.

## Last Updated

- 2026-02-11

## Change Log

- 2026-02-11: Initialized memory file and recorded key decisions.
- 2026-02-11: Added stack weights and first-release deployment constraint.
- 2026-02-11: Added stack evaluation template.
- 2026-02-11: Decided on single-stack Rust based on performance priority and industry trends.

## Current Status

- Overview complete and updated in requirements/000-overview.md
- Phased plan in requirements/001-phased-plan.md (Phase 0: ~3 weeks, Phase 1 focus, Phase 2 revisit after lessons learned)
- Requirements initial draft created in requirements/010-requirements.md
- Design document populated with Phase 1 architecture in requirements/020-design.md
- Stack evaluation and decision recorded in requirements/030-stack-evaluation.md (Rust selected)
- Benchmark harness plan created in requirements/040-benchmark-harness.md
- Ready to begin Phase 0: benchmark implementation and validation

## Key Decisions

- Target usage: local or single Docker instance, not distributed for first release
- Determinism: probabilistic, converges with sample size
- Distribution accuracy tolerance: within 10% at >=100 samples, clearer at ~1,000
- Soak reliability: 8 to 12 hours without crash or corruption
- First release UI: add endpoints, configure behavior, import/export config
- Logging: plain text with timestamps; levels DEBUG/INFO/WARN/ERROR; reduce verbosity under load
- CPU warnings: >80% for 10s (warn), 90% elevated, 100% critical
- Config format: YAML or JSON with explicit override rules
- Stack evaluation weights: Performance 45, Safety 20, Operability 15, Ecosystem 10, Dev velocity 5, Hiring 5
- First release scope: single executable or single Docker image; no distributed deployment
- Stack decision: single-stack Rust for engine and control plane (maximize performance headroom and simplify deployment; UI iteration speed is secondary)

## Open Items

- Phase 0: implement benchmark harness and run validation tests (~3 weeks)
- Configuration schema finalization (after benchmark validation)
- Phase 1 implementation (after Phase 0 complete)
- Phase 2 scope revisit (after Phase 1 lessons learned)
