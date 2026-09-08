# Swarms and Combat Feedback Implementation Plan

> **For agentic workers:** Use executing-plans to implement this plan task by task. Use requesting-code-review for independent review before publication.

**Goal:** Deliver DRO-7's approved three-minute swarm encounter, feedback, and measured native performance in a PR against main.

**Architecture:** Combat owns a run-local encounter clock, a bounded authored wave schedule, warning entities as cap reservations, and frame-local combat outcomes. Presentation consumes outcomes for reusable effects and HUD; enemy separation feeds the existing physical pilot. A command-line validation harness measures rendered workloads separately from ordinary gameplay.

**Tech Stack:** Existing Rust nightly, Bevy 0.19.1, headless ECS tests, native macOS, GitHub CLI.

## Global constraints

Preserve scout controls and collision/flight integration. Survival lasts 180 seconds, with restart before fatal damage before success. Pending warnings count toward the default cap of 30. Warnings last 0.75 seconds and require 120 units of player clearance. Spawns skipped for safety, capacity, or hitches create no debt. Keep downstream energy, upgrades, rewards, campaign flows, and audio out of scope. Numeric tuning is provisional and actual results go in docs/playtests.md.

## Milestone 1 — Encounter rules and safe timed waves

Files: src/combat.rs, src/combat/waves.rs, src/combat/lifecycle.rs, src/combat/enemies.rs, src/game.rs, src/combat/tests.rs, src/combat/wave_tests.rs.

Interfaces: WaveConfig supplies bursts, cap, warning duration and clearance; Encounter holds elapsed time, kills, next burst and deterministic candidate cursor; SpawnWarning entities reserve positions until activation. Existing spawn logic becomes spawn_enemy(commands, config, position, target).

- [x] Run baseline `cargo test --locked` and create the feature branch.
- [x] Add a failing real-plugin test asserting startup contains zero enemies before the first warning. Update old fixed-encounter tests to explicitly create their three-enemy fixture, preserving their original targeting, flight and collision coverage.
- [x] Add controlled-time regressions for first warning/activation, live-plus-reserved cap, cap saturation without spawn debt, unsafe activation cancellation, long-frame latest-burst handling, 180-second success, death precedence and restart cleanup. Drive real systems with the existing `step(&mut app, dt, keys)` helper.
- [x] Implement authored bursts at 3/15/27/39 (3 enemies), 60/70/80/90 (4), and 120/128/136/144/152 (5). Bound perimeter candidate search; use rotated bounds at reservation and activation; cancel invalid warnings. Clamp run time to duration and finish after damage resolution.
- [x] Run the focused tests, full suite, formatting and Clippy; commit `feat: add capped survival waves and encounter lifecycle`.

## Milestone 2 — Separation and combat feedback

Files: src/combat/enemies.rs, src/combat/weapon.rs, src/combat/lifecycle.rs, src/combat/feedback.rs, src/combat/scene.rs, src/combat/feedback_tests.rs, src/combat/enemy_flight_tests.rs.

Interfaces: CombatOutcomes holds frame-local hit/kill/player-damage outcomes. Gameplay increments Encounter.kills on lethal resolution once. FeedbackConfig bounds effect count and lifetimes; scene systems own reusable normal/hit/warning/kill material handles. Separation acceleration enters the physical pilot, never directly changing positions.

- [x] Add failing tests for two lethal shots producing one kill, blocked contact producing no extra damage cue, material isolation, capped effects/expiry, warning visuals, HUD lulls/results and complete reset without asset growth.
- [x] Add physical separation tests covering coincident starts, gradual spreading, bounds and maintained pursuit.
- [x] Implement bounded outcomes-driven feedback and local separation, reusing meshes/materials. Show hull, time remaining, hostiles, kills, phase/lull and terminal prompts.
- [x] Run focused tests and retained regressions; commit `feat: add readable swarm steering and combat feedback`.

## Milestone 3 — Native validation, tuning and publication

Files: src/combat/validation.rs, src/main.rs, README.md, docs/playtests.md, this plan.

Interfaces: Explicit validation CLI options enable a repeatable native stress workload and frame-time sampling. Ordinary launch retains the approved scenario. Reports include actual enemy-count range, frame percentiles and hitches after warm-up; harness overrides are clearly identified.

- [x] Add meaningful tests for percentile summaries and validation options/workload invariants before implementing them.
- [x] Implement a bounded measurement window using real wall-clock frame durations, with median/p95/p99 and >33.3 ms counts. Support a maintained 150-enemy stress workload with physical pursuit, shots and feedback. Document any player invulnerability or refill override.
- [ ] Run `cargo fmt --check`, `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings`, and native `cargo dev` validation. Exercise the complete encounter, death, restart, warning/effect readability and performance. Record exact machine/build/resolution and remaining human-control limitations.
- [ ] Tune only when observations show a need. Record any 150-enemy performance fallback honestly, separately from playable balance.
- [ ] Request an independent source review while finishing native measurements, address actionable findings, and rerun affected checks.
- [ ] Update README and playtests, complete this checklist, and commit the validation milestone.
- [ ] Verify final diff and clean tracked state, push `codex/dro-7-swarms-and-feedback`, and create a PR against main with results and limitations. Do not merge.
