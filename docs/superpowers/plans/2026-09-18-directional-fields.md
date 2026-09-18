# Directional Fields and Hull Repair Implementation Plan

> Execute inline with the executing-plans skill, following test-first cycles.

**Goal:** Deliver DRO-38 as two isolated, selectable environment scenarios.
**Architecture:** Catalog configures a world environment resource. Player flight
reads it per substep; catalog systems reset/tick/consume repair and present cues.
**Tech Stack:** Existing Rust/Bevy, no dependencies.

## Constraints

Campaign unchanged; one 35-hull repair charge per round; east travel 1.4x, west
0.6x, perpendicular/vertical 1x; overlap never stacks; 6s on/4s off cycle; pause
and reset use existing gameplay phases. Human acceptance remains deferred.

## Tasks

- [x] Add failing integration tests in `src/combat/catalog_tests.rs` selecting
  environment practice (index 12), asserting active fixtures, repair usage,
  full-hull preservation, pause, restart and scenario isolation. Run
  `cargo test --locked environment` and confirm missing behavior fails.
- [x] Add `src/world/environment.rs` with `Environment` resource, field bounds,
  bounded travel scale and repair state. Configure/reset/tick/repair from
  `src/combat/catalog/environment.rs`; append scenarios in catalog.rs. Add
  field math tests and implement minimum production behavior.
- [x] Integrate field sampling in `src/arena.rs` and `src/arena/flight.rs`, before
  swept terrain resolution, retaining the existing entry points for AI and
  ordinary flight. Validate production movement, exit, collision and 30/60/120
  Hz comparisons. Commit gameplay increment after targeted tests pass.
- [x] Add cached mesh cues and compact status text in
  `src/combat/catalog/environment_scene.rs`. Add opt-in native fixture in
  `src/combat/catalog/environment_validation.rs` and register via
  `DRONE_ENVIRONMENT_SMOKE`. Run `cargo dev -- --validate catalog` at both
  supported sizes; inspect screenshots and fix any presentation defects.
- [ ] Run `cargo fmt --check`, `cargo test --locked`, and
  `cargo clippy --locked --all-targets -- -D warnings`. Record evidence and
  limitations in `docs/playtests/dro-38-environment.md`, `docs/playtests.md`
  and README. Commit verified evidence, review diff, push, open PR to main.
