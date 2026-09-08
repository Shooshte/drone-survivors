# Drone flight controls implementation plan

> **For agentic workers:** Use subagent-driven-development for the flight implementation and an independent review. Checkboxes track completion.

**Goal:** Implement the approved heading-relative, thrust-driven keyboard flight controls.

**Architecture:** Keep input, attitude, forces, and containment in focused helpers under ArenaPlugin. Store persistent velocity/heading/tilt on the player, with bounded simulation substeps before combat. Share rotated bounds with contact damage and presentation.

**Tech stack:** Rust nightly, existing Bevy 0.19.1, no additional dependencies.

## Global constraints

- Follow `docs/superpowers/specs/2026-09-08-drone-flight-controls-design.md` and its tuning defaults.
- W/S pitch, A/D yaw, Q/E bank, Space/Shift boost/reduce thrust; aliases and opposing inputs cancel independently.
- Releasing tilt levels smoothly, but preserves linear momentum. Neutral thrust balances gravity only when level; do not compensate for altitude loss.
- Maximum horizontal speed is 240 world units/second. Total tilt is limited to 30 degrees.
- Keep the full rotated drone inside the arena; remove only contact-normal inward velocity.
- Reset clears all flight state and takes priority; death freezes simulation.
- Preserve automatic aiming, camera, arena, model asset, and encounter rules.
- Worktree: `.worktrees/drone-flight-controls`, branch `codex/drone-flight-controls`.
- Use `CARGO_TARGET_DIR=/Users/shooshte/projects/drone-survivors/target` for Cargo commands to reuse compiled dependencies.

## Task 1: Flight simulation and integration

Files:
- Create `src/arena/flight.rs`: configuration, persistent flight state, input interpretation, attitude, force integration, speed limiting, rotated bounds, and containment.
- Modify `src/arena.rs`: spawn flight state, configure resource, replace direct movement while preserving reset/combat ordering.
- Modify `src/arena/scene.rs`: updated legend and rotated-bound altitude guide.
- Modify `src/combat/lifecycle.rs`: rotated bounds for contact damage.
- Update `src/arena/tests.rs`; add focused tests under `src/arena/flight/` if needed.
- Update `src/combat/tests.rs`: moving/dead reset state, rotated contact, and heading-independent aiming.

Interfaces:
- `DroneFlight` component stores world-space velocity, heading, and heading-local tilt.
- `FlightConfig` resource holds all spec tuning values.
- `drone_world_half_extents(rotation: Quat) -> Vec3`, re-exported from `arena`, provides the common rotated envelope.
- Existing `move_drone`, `spawn_drone`, and `ArenaPlugin` remain integration entry points.

- [x] Add behavioral regressions before implementation. Initial assertions can use existing Transform APIs so the old implementation compiles and demonstrably fails:

```rust
step(&mut app, &[KeyCode::KeyA], 0.25);
let transform = app.world().get::<Transform>(drone).unwrap();
assert_ne!(transform.rotation, Quat::IDENTITY);
near(transform.translation, START);
```

```rust
step(&mut app, &[KeyCode::KeyQ], 0.5);
assert!(position(&app, drone).x < START.x);
assert!(position(&app, drone).y < START.y);
```

- [x] Run `cargo test --locked arena::tests` and record expected failures from missing yaw, banking, drift, and altitude loss. Replace tests for superseded immediate movement; retain scene/asset/camera checks.
- [x] Implement persistent simulation with substeps no longer than 1/120 second, signed heading-local tilt, local-up thrust, gravity, stable drag, speed taper and clamp. Shared world bounds are the sum of absolute rotated basis vectors weighted by local half-extents:

```rust
(rotation * Vec3::X).abs() * DRONE_HALF_EXTENTS.x
    + (rotation * Vec3::Y).abs() * DRONE_HALF_EXTENTS.y
    + (rotation * Vec3::Z).abs() * DRONE_HALF_EXTENTS.z
```

- [x] Add regressions for all aliases/opposites, heading-relative acceleration, combined tilt limits, axis-independent leveling, hover/descent/thrust release, drift/braking, speed taper without lost steering, all 26 boundary directions, contact departure, reset/death, and comparable 30/60/120/144 Hz trajectories including a long frame.
- [x] Verify `cargo test --locked`, `cargo fmt --check`, and `cargo clippy --all-targets --locked -- -D warnings`; fix failures and self-review against the spec.

## Task 2: Documentation and native validation

Files: `README.md`, `docs/playtests.md`, approved spec status, this plan.

- [x] Replace README movement instructions with the implemented bindings and explain drift, altitude loss, counter-tilt braking, and thrust release.
- [x] Add current playtest steps before historical entries. Mark earlier control instructions as historical.
- [x] Build `cargo build --locked --features bevy/dynamic_linking`, launch the native arena, verify legend/model/orientation/reset where UI tools permit, and record exact observations and limitations.
- [x] Review the complete diff independently; resolve actionable findings with covering tests.
- [x] Run final formatting/tests/Clippy and diff checks, mark plan complete, and commit the finished implementation.
- [ ] Push `codex/drone-flight-controls`; use `gh pr create --base main --head codex/drone-flight-controls --body-file <prepared-file>` and verify the returned PR state/base/head.

## Validation record

- Baseline: 32 tests passed on the isolated branch before implementation.

- Final source: 42 tests passed; formatting, Clippy with warnings denied, dynamic native build, and diff checks passed.
- Independent review: spec compliance and code quality passed with no actionable findings.
- Native rendering, resized legend, restart, and exit verified; sustained keyboard feel remains a human playtest item, detailed in `docs/playtests.md`.
