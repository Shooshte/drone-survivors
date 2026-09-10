# Charger Depletion Implementation Plan

> **For agentic workers:** Use subagent-driven-development for bounded implementation and review, with the controller handling integration and native validation. Track completion here.

**Goal:** Finite, readable chargers that recover only after sustained absence.

**Architecture:** Store reserve/recovery state beside charging geometry. Compute chronological power flow in a focused pure calculation module, stage it through PowerFrame, and commit after outcome resolution. Presentation reads committed state only.

**Tech Stack:** Rust, Bevy 0.19.1, existing headless ECS tests and native validation.

## Global Constraints

- Each existing charging field starts with an independent 200-energy reserve.
- Delivery remains capped at the existing 25 energy/second.
- After eight uninterrupted active-gameplay seconds outside that field's existing 3D charging volume, it recovers 10 reserve/second, capped at 200.
- Choosing, Dead and Survived freeze reserve and recovery clocks.
- R restores both full reserves and clears absence timers.
- Enemy behavior, waves, XP, scout handling, arena geometry, module costs, and campaign systems stay outside this change.
- Use CARGO_TARGET_DIR=/Users/shooshte/projects/drone-survivors/target for commands in this worktree.

## Task 1: Reserve accounting and lifecycle

Files: src/energy.rs; new src/energy/flow.rs; src/energy/tests.rs; src/combat/energy_tests.rs.
Interfaces: ChargerConfig resource { capacity: f64, recovery_delay: f64, recovery_rate: f64 }; ChargerReserve component { remaining: f64, away_seconds: f64, occupied: bool }. ChargingNode requires ChargerReserve; setup/reset honor ChargerConfig. PowerFrame stages a vector of entity/reserve pairs. Energy.charging denotes the currently available nonempty source, including at zero delta time. The pure solver advances battery, modules and occupied reserves through full/empty boundaries without accumulating wasted supply.

- [ ] Write a behavior test using existing APIs: camp at the left field with overdrive for 31 seconds; assert battery zero and module off. Run it and observe existing infinite-supply behavior fail.
- [ ] Add reserve accounting tests: full battery consumes only module demand; all-off full battery consumes none; empty battery refills; overlaps share one 25/s ceiling and hand off on source exhaustion; all occupied fields cannot recover; uninterrupted absence and reentry; partial refill; 30/60/120 Hz and long-update equivalence including shield powered time.
- [ ] Implement chronological integration in flow.rs; stage recovery and supply changes together; preserve pre-charge activation and post-damage commit.
- [ ] Extend integrated pause/reset/fatal/survival tests to reserve state. Adapt old infinite-supply assumptions explicitly, retaining their independent assertions.
- [ ] Run focused energy/module regressions, then full locked tests and Clippy; commit gameplay and tests.
- [ ] Review spec compliance and implementation quality; resolve findings before closing task.

## Task 2: Charger status and world reserve indicators

Files: src/energy/scene.rs; optionally new src/energy/charger_scene.rs.
Consumes ChargerConfig and ChargerReserve from Task 1.

- [ ] Add failing presentation tests for LEFT/RIGHT reserve status, occupied depletion, away-delay countdown, recovery, and paused presentation.
- [ ] Implement two compact HUD rows plus world fill indicators that shrink as reserve decreases. Keep an empty outline visible. Include readable text and status alongside color.
- [ ] Derive battery charging text from availability; never advertise recharge at a depleted field. Keep existing battery/module HUD behavior and restart asset reuse.
- [ ] Run focused scene tests; capture normal-size and 640x480 native scenes and correct overlaps/clipping; commit.
- [ ] Review task spec compliance and quality; resolve findings.

## Task 3: Encounter comparison and delivery

Files: src/combat/camping_tests.rs or new charger diagnostic module; docs/playtests.md; new docs/playtests/dro-32-charger-depletion.md; README.md.

- [ ] Preserve the existing ten-scenario diagnostic output from main in /tmp/dro32-baseline.log. Freeze its historical infinite supply fixture so it remains reproducible.
- [ ] Compare four stationary current-wave builds and a moving pilot with finite versus effectively unlimited test-only reserves. Use real terrain, normal flight, damage, guarded choices and normal XP.
- [ ] Record actual reserve delivery, powered seconds, charging visits, choice times and outcomes. Moving pilot alternates fields with deliberate power conservation; never teleport or replenish health/XP.
- [ ] Run native depletion/recovery/restart smoke and both sizes. Document that scripted outcomes do not establish human tactical acceptance.
- [ ] Run cargo fmt --check, cargo test --locked, cargo clippy --all-targets --locked -- -D warnings and opt-in diagnostics. Commit evidence and documentation.
- [ ] Final whole-branch review; fix important findings and rerun affected checks. Push codex/dro-32-charger-depletion; open PR against main with honest validation limitations.
