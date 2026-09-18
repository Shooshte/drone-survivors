# Control enemies implementation plan

**Goal:** Implement DRO-36 with catalog-only slowing beams and jammers.
**Architecture:** Shared ranged attack component, frame-derived slow resource,
module lock timers, existing enemy flight and power accounting.
**Stack:** Rust, Bevy 0.19.1, existing Cargo toolchain.

## Constraints

Follow the accompanying control-enemies design. Keep campaign/save behavior and
four equipped slots unchanged. All durations use active gameplay time.

## Steps

- [x] Add failing production integration tests in `src/combat/control_tests.rs`
  and module tests for lock rejection, expiry, power and shielding. Use existing
  ArenaPlugin/CombatPlugin test harness with explicit `Time` increments.
- [x] Implement `src/combat/control.rs`: `ControlAttack` phases and
  `ControlEffects` aggregate, pre-movement update and reset. Add Slower/Jammer
  variants to EnemyKind; attach components at spawn. Use existing chase pilot
  with an attack anchor. Add module disable/grace state and derive movement slow
  without mutating baseline configs. Run targeted tests, then commit behavior.
- [x] Implement `src/combat/control_scene.rs` with cached meshes/materials,
  child attack cues and catalog-only status text. Extend module rows with lock
  duration. Add standalone and mixed catalog scenarios and selection tests.
  Run tests and commit presentation/scenarios.
- [x] Add native `DRONE_CONTROL_SMOKE` fixture in catalog validation. Check
  windup, beam, jammer lock/recovery, pause/reset, cover and scenario selection;
  capture both window sizes. Record evidence in README and docs/playtests.md.
- [x] Run `cargo fmt --check`, `cargo test --locked`,
  `cargo clippy --locked --all-targets -- -D warnings`, review diff, commit,
  push branch and open PR against main.
