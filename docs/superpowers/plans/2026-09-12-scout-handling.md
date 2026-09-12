# Scout Handling Implementation Plan

> Execute in the existing isolated worktree, using test-driven development and an independent review before delivery.

**Goal:** Deliver approved coordinated Q/E turns and responsive player-only acceleration/braking.
**Architecture:** Keep the shared rotor integrator; express assistance and counter-tilt response in FlightConfig with explicit legacy enemy values. Apply heading assistance per physics substep without rotating velocity.
**Tech Stack:** Rust, Bevy 0.19.1, existing Cargo setup.

## Global constraints

Preserve enemy tuning, arena geometry, vertical flight, speed caps, upgrade costs,
pause, reset and swept collision. No new dependency. All tuning is provisional.

## Task 1: response and regression coverage

- [ ] Create src/arena/handling_tests.rs, registered by src/arena.rs. Use the real
  ArenaPlugin and keyboard input in an enlarged test-only arena. Measure forward
  launch, braking from 300 units/s with opposite pitch, mirrored banked curves,
  direct counter-yaw, release and reversal at 30/60/120 Hz. Record response times
  and stop distances for baseline, mobility and armor/rounds.
- [ ] Run `cargo test --locked handling -- --nocapture`; confirm new behavior fails.
- [ ] Update FlightConfig and attitude integration in src/arena/flight.rs. Give
  enemies explicit unassisted defaults in src/combat.rs. Extend angular upgrade
  scaling in src/upgrades.rs if its config application requires it.
- [ ] Adjust superseded straight-bank assertions, preserving directional, physical
  momentum and envelope checks. Run all tests and commit the feature.

## Task 2: native validation and delivery

- [ ] Run `cargo fmt --check`, `cargo test --locked`, and
  `cargo clippy --locked --all-targets -- -D warnings`.
- [ ] Build/run `cargo dev -- --validate routes --seconds 40` and a normal
  encounter using the existing manual validation mode; inspect native rendering.
- [ ] Document controls, measurements, native observations and remaining human
  feel checks in README.md and docs/playtests/dro-29-scout-handling.md, linked
  from docs/playtests.md. Update in-game control text to say Bank + turn.
- [ ] Request independent code review, resolve material findings, and rerun
  affected checks. Commit documentation and fixes separately as appropriate.
- [ ] Push codex/dro-29-scout-handling and open a PR against main. Clearly disclose
  human playtest observations still needed instead of marking that gate passed.
