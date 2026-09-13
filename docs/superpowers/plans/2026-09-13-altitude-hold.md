# Altitude Hold Implementation Plan

> Execute inline using executing-plans and test-driven-development.

**Goal:** Complete DRO-33 with automatic height hold and unchanged horizontal handling.

**Architecture:** Keyboard input selects hold or tilt-compensated vertical thrust;
AI selects rotor physics. Shared movement preserves collision resolution.

**Tech stack:** Rust, Bevy 0.19.1, existing native validation and Rust tests.

## Global constraints

No new dependencies, enemy tuning, encounter tuning, or horizontal handling changes.
Branch: codex/dro-33-altitude-hold, based on origin/main at da642ce.

## Task 1 — Flight behavior and regressions

- [ ] Update superseded altitude-loss/inertia assertions in src/arena/tests.rs.
  Add src/arena/altitude_tests.rs through `#[cfg(test)] mod altitude_tests;`.
  Test neutral multi-axis maneuvers, release, aliases, manual vertical trajectory,
  horizontal equivalence, contacts, and real restart. Use actual keyboard input.
- [ ] Run `cargo test --locked altitude` and see height/velocity assertions fail.
- [ ] In src/arena/flight.rs add `VerticalControl::{RotorThrust, AltitudeHold,
  TiltCompensated}` and a `vertical: VerticalControl` field on FlightInput.
  Read neutral vertical axis as AltitudeHold; nonzero as TiltCompensated.
  Preserve `thrust` for horizontal forces. In acceleration, compensated Y is
  `config.gravity * (input.thrust - 1.)`. In integrate, hold skips Y displacement
  and sets `self.velocity.y = 0.` before ordinary collision resolution.
  Update explicit raw-physics FlightInput fixtures and enemy inputs with
  `vertical: VerticalControl::RotorThrust`.
- [ ] Run full `cargo test --locked`; diagnose affected assumptions and retain
  existing physical collision and AI coverage with explicit raw inputs.
- [ ] Commit passing flight behavior and regressions.

## Task 2 — Documentation and native verification

- [ ] Change README and src/arena/scene.rs help to Space ascend / Shift descend,
  automatic altitude hold, horizontal drift on release and collision clearance.
- [ ] Build native game; exercise hover, pitch/bank, release from ascent/descent,
  opposing keys, boundary contacts and R using native UI control. Record actual
  observations and limitations in docs/playtests.md.
- [ ] Run `cargo fmt --check`, `cargo test --locked`,
  `cargo clippy --locked --all-targets -- -D warnings` and review final diff.
- [ ] Commit documentation/evidence, push, and create PR against main with ticket
  link, behavior, validation and any remaining manual subjective-playtest limits.
