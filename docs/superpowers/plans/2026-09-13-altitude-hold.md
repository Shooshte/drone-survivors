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

- [x] Update superseded altitude-loss/inertia assertions in src/arena/tests.rs.
  Add src/arena/altitude_tests.rs through `#[cfg(test)] mod altitude_tests;`.
  Test neutral multi-axis maneuvers, release, aliases, manual vertical trajectory,
  horizontal equivalence, contacts, and real restart. Use actual keyboard input.
- [x] Run `cargo test --locked altitude` and see height/velocity assertions fail.
- [x] In src/arena/flight.rs add `VerticalControl::{RotorThrust, AltitudeHold,
  TiltCompensated}` and a `vertical: VerticalControl` field on FlightInput.
  Read neutral vertical axis as AltitudeHold; nonzero as TiltCompensated.
  Preserve `thrust` for horizontal forces. In acceleration, compensated Y is
  `config.gravity * (input.thrust - 1.)`. In integrate, hold skips Y displacement
  and sets `self.velocity.y = 0.` before ordinary collision resolution.
  Update explicit raw-physics FlightInput fixtures and enemy inputs with
  `vertical: VerticalControl::RotorThrust`.
- [x] Run full `cargo test --locked`; diagnose affected assumptions and retain
  existing physical collision and AI coverage with explicit raw inputs.
- [x] Commit passing flight behavior and regressions.

## Task 2 — Documentation and native verification

- [x] Change README and src/arena/scene.rs help to Space ascend / Shift descend,
  automatic altitude hold, horizontal drift on release and collision clearance.
- [x] Build native game and record UI smoke observations in docs/playtests.md.
  Held-input UI delivery was unreliable, so sustained human flight remains
  pending. Existing native route pilot completed all four legs without damage;
  deterministic regressions cover ascent/descent/release, opposing keys and contacts.
- [x] Run `cargo fmt --check`, `cargo test --locked`,
  `cargo clippy --locked --all-targets -- -D warnings` and review final diff.
- [x] Prepare documentation/evidence for the final commit and PR against main,
  including ticket link, behavior, validation and the sustained manual-flight limit.
  The authorized final delivery is to push this branch and open the PR.
