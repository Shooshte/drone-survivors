# Scout response and enemy flight implementation plan

> **For agentic workers:** Use subagent-driven-development for source implementation and independent review; the parent handles documentation and native validation.

**Goal:** Give the scout prompt acceleration and handling, and make enemies pursue through the same physical flight model.

**Architecture:** Generalize the existing flight integrator's control/bounds inputs, retaining the keyboard adapter. Add an AI pilot and a slower enemy profile. Share rotated collision envelopes across flight and combat.

**Tech stack:** Existing Rust nightly and Bevy 0.19.1; no additional dependencies.

## Global constraints

- Follow `docs/superpowers/specs/2026-09-08-scout-response-and-enemy-flight-design.md`.
- Keep this worktree and `codex/drone-flight-controls`; update existing PR #5.
- Preserve bindings, player altitude loss, momentum, automatic leveling, reset/death, and weapon/damage rules.
- All flight uses the same world gravity and bounded physical integration.
- Cargo commands use `CARGO_TARGET_DIR=/Users/shooshte/projects/drone-survivors/target`.

## Task 1: Test and implement scout response and common enemy flight

Files: `src/arena/flight.rs`, `src/arena.rs`, `src/arena/tests.rs`, `src/combat.rs`,
`src/combat/enemies.rs`, `src/combat/lifecycle.rs`, `src/combat/weapon.rs`, and
`src/combat/tests.rs`; extract AI tests/helpers into focused files as needed.

- [ ] Add failing scout launch/response tests and enemy acceleration/momentum tests before changing behavior.
- [ ] Tune scout to the spec values; update existing assertions to test the new defaults while retaining old behavior coverage.
- [ ] Expose analog controls and actor-local bounds in shared flight integration. Preserve keyboard normalization and canceling opposite inputs.
- [ ] Spawn enemy flight state and replace constant-speed chase with a bounded-step AI pilot and the enemy flight profile.
- [ ] Apply consistent rotated enemy bounds to arena/contact/projectile collision; retain previous-position sweeps.
- [ ] Add behavioral pursuit, altitude, turning/braking, frame-rate, contact, death/reset, and default two-second evasion regressions.
- [ ] Run full tests, formatting, and Clippy with warnings denied; resolve failures and self-review.

## Task 2: Review, native verification, and delivery

Files: `README.md`, `docs/playtests.md`, original spec tuning references, this plan.

- [ ] Update the current tuning and enemy behavior documentation; preserve historical playtest results as historical.
- [ ] Build and run native game; inspect enemy attitude/movement and controls, and record limitations.
- [ ] Independently review the complete follow-up diff; resolve actionable findings with covering tests.
- [ ] Run final verification, commit, push, and update PR #5's title/body around the resulting scout and pursuit behavior.
