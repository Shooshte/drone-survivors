# Catalog Combination Validation Implementation Plan

> **For agentic workers:** Use subagent-driven-development for the independent comparison probe and requesting-code-review before delivery.

**Goal:** Complete DRO-41's repeatable agent combination evidence in the isolated arena.

**Architecture:** One combined scenario uses existing catalog mechanics. A test-only
probe drives production controls and records measurements; an opt-in native fixture
checks presentation. Keep evidence separate from human acceptance.

**Tech Stack:** Rust, Bevy 0.19.1, Cargo tests and native cargo dev.

## Global Constraints

- Catalog only: no campaign saves, spawns, shops or offer expansion.
- Seven enemy kinds, six modules, twelve upgrades; at most four upgrades per run.
- Combined catalog scenario index 14: all seven kinds, bursts (1,7), (16,7), (31,7), existing environment enabled.
- Normal resource costs, damage, finite chargers and resets; synthetic input/preview XP disclosed.
- Balance changes require measured justification; human acceptance remains pending.

### Task 1: Combined scenario and native presentation

Files: src/combat/catalog.rs, src/combat/catalog_tests.rs,
src/combat/catalog/combination_validation.rs.

- [ ] Add failing selector test: cycle to Combined catalog, launch, assert all seven roster kinds and environment, restart/return reset, no campaign state.
- [ ] Add scenario using existing waves/roster/environment; update old scenario-wrap expectation.
- [ ] Add DRONE_COMBINATION_SMOKE fixture using keyboard configuration and normal preview choices; run a combined round, restart, return; check text bounds and capture setup/preview/HUD.
- [ ] Run native fixture at both supported window sizes, inspect captures, commit coherent increment.

### Task 2: Measured comparison probe

Files: src/combat/catalog/combination_tests.rs, one test-module registration in catalog_tests.rs;
evidence and report in /tmp/dro-41-probe-report.md for final documentation.

- [ ] Use production catalog app at fixed 30 Hz with real geometry, hazard, resources and controls.
- [ ] Exercise Combined catalog with empty/single-module controls and contrasting four-slot/four-card builds. Cover all six modules and twelve upgrades across the matrix.
- [ ] Compare stationary and moving routes (including field/repair traversal), report active time/outcome/hull/kills/power, observed kinds and environment usage. Keep naturally earned offers and documented previews within four opportunities.
- [ ] Include controlled comparisons for tradeoffs that aggregate outcomes cannot establish; use existing meaningful regressions as supporting counterplay evidence.
- [ ] Run opt-in catalog_combination_probe and record exact command/results/limits. Reproduce any defect with a failing regression before changing production behavior.
- [ ] Self-review probe reliability and commit its independent files.

### Task 3: Evidence, review and delivery

Files: README.md, docs/playtests.md, docs/playtests/dro-41-catalog-validation.md,
docs/playtests/evidence/dro-41/, selected images, this plan.

- [ ] Record measured results and limits, responses for all enemies, uses for six modules, twelve benefit/drawback comparisons, tuning decisions and pending human checks.
- [ ] Run cargo fmt --check, cargo test --locked, cargo clippy --locked --all-targets -- -D warnings, git diff --check.
- [ ] Independent review; resolve findings and rerun affected checks.
- [ ] Commit, push codex/dro-41-catalog-validation and open PR against main with Linear link.
