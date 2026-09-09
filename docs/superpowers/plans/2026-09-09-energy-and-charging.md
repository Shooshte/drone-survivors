# Energy and Charging Implementation Plan

> **For agentic workers:** Execute tasks in order with test-driven development and review at the completed feature milestone.

**Goal:** Deliver DRO-8's approved battery, key-1 weapon overdrive, two charging volumes, and readable feedback.

**Architecture:** EnergyPlugin owns energy and charging rules; EnergyScenePlugin owns nodes and HUD. Combat calls the energy update after outcome resolution and consumes the effective firing multiplier. Existing arena movement remains unchanged.

**Tech Stack:** Rust, Bevy 0.19.1, existing Cargo dependencies and native macOS.

## Global Constraints

- Follow docs/superpowers/specs/2026-09-09-energy-and-charging-design.md.
- Capacity 100, drain 10/s, recharge 25/s, activation threshold 10, multiplier 2.
- Key 1 toggles; restart wins; terminal state freezes; base flight/fire always work.
- Two radius-90, height-160 cylinders at x ±280, z 0; center containment inclusive.
- Keep user-owned untracked files out of commits.

## Task 1: Battery and charging rules

Files: create src/energy.rs and src/energy/tests.rs; register module in src/main.rs.
Interface: EnergyConfig tuning, Energy resource (current, overdrive, charging,
rejected_for), ChargingNode component with contains(Vec3), EnergyPlugin,
and update system accepting time, input, drone transform, phase and nodes.

- [x] Establish baseline with `cargo test --locked`.
- [x] Write headless ECS tests for depletion, refill, geometry, both nodes,
  overlapping nodes, input threshold/holds, manual recovery and terminal/reset.
  Start with a no-op update and watch behavioral assertions fail.
- [x] Implement combined-rate clamping and persistent nodes. For example:
  `current = (current + (recharge - drain) * dt).clamp(0., capacity);`
  `if current == 0. { overdrive = false; }`
- [x] Run `cargo test --locked energy::tests` and fix rule failures.
- [x] Commit the tested rules and spec/plan.

## Task 2: Weapon and encounter integration

Files: modify src/combat.rs, src/combat/weapon.rs, src/main.rs;
create src/combat/energy_tests.rs and register from combat module.
Interface: Energy::fire_multiplier(&EnergyConfig) -> f64; Weapon tracks previous
cadence so `remaining / old_interval * new_interval` preserves cooldown progress.

- [x] Add failing tests proving double cadence, ordinary shots and movement at
  zero, no extra shots from toggles, restart precedence and terminal ordering.
- [x] Place energy update after waves::finish and before weapon::fire in combat's
  ordered chain. Reset belongs to GameplaySet::Reset. Install EnergyPlugin with
  CombatPlugin so existing headless combat fixtures exercise integration.
- [x] Retain all existing firing protections and projectile attributes; only
  change effective interval and retime outstanding cooldown on transitions.
- [x] Run `cargo test --locked` and commit combat integration.

## Task 3: Field visuals and HUD

Files: create src/energy/scene.rs; modify src/main.rs and README.md.
Interface: EnergyScenePlugin reads Energy/Config/ChargingNode and owns persistent
visual handles, an energy fill node and energy status text.

- [x] Render identical translucent cylinders with clear floor/top rings and
  occupied-field highlighting. Use assets allocated at setup, not each frame.
- [x] Place energy HUD below combat text, with numeric battery and net rate,
  ON/OFF/EMPTY and rejected-activation explanation; include key 1 hint.
- [x] Verify HUD transitions and no asset/entity growth over repeated restarts.
- [x] Run formatting, tests and Clippy; commit presentation and controls docs.

## Task 4: Native validation, review and delivery

Files: docs/playtests.md and any fixes supported by review/validation evidence.

Result: 87 tests, formatting, Clippy and native checks passed; see docs/playtests.md.

- [x] Run `cargo dev`; inspect fields/HUD, key toggle, depletion, recovery and
  restart. Use repeatable native validation for encounter compatibility.
- [x] Run `cargo fmt --check`, `cargo test --locked`, and
  `cargo clippy --all-targets --locked -- -D warnings`.
- [x] Request read-only independent code review of origin/main..HEAD while
  documenting native results. Fix actionable issues with regression tests.
- [x] Record actual results and limits in docs/playtests.md; commit.
Delivery command: push codex/dro-8-energy-and-charging and create PR against main with
  `gh pr create --base main --body-file <prepared-file>`.
