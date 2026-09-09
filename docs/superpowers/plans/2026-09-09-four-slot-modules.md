# Four-slot Modules Implementation Plan

> Execute with subagent-driven-development for the bounded rocket task and an independent final review; integrate the shared power and flight changes locally.

**Goal:** Four independently powered prototypes with meaningful energy tradeoffs.
**Architecture:** `modules.rs` owns validated slots, tuning, toggles and shield state. `energy.rs` owns battery/charging and a staged power update. Combat and flight consume module effects; presentation shows per-slot states. Rockets reuse the swept projectile collision path.
**Tech Stack:** Existing Rust/Bevy; no new dependencies.

## Global constraints

Follow `docs/superpowers/specs/2026-09-09-four-slot-modules-design.md`. Defaults: drain 10/8/8/10, activation threshold 10, shield one block and 5 powered seconds, mobility horizontal acceleration/cap 1.25x, rockets 20 damage/70 radius/2s interval. Preserve baseline fire, flight, terminal freeze, restart and charging. All slots interchangeable, no duplicate types. No campaign/loadout UI.

## Task 1 — Slots, power and shield state

Files: create `src/modules.rs`, `src/modules/tests.rs`; modify `src/main.rs`, `src/energy.rs`, energy tests and existing overdrive consumers.

- [x] Add failing ECS tests for each key's independent drain, all-enabled net flow, held input, low-energy rejection and depletion. Run `cargo test --locked module` and observe missing behavior.
- [x] Implement `ModuleKind::{Overdrive,Shield,Mobility,Rocket}`, `ModuleConfig`, validated `Loadout([Option<ModuleKind>;4])`, and `Modules` state. `Modules::active(kind)` queries enabled occupied slots; `drain(config)` sums enabled rates. `Modules::block(config)` consumes a shield block and starts recharge. Toggling never alters charge progress.
- [x] Stage a `PowerFrame { energy: Energy, modules: Modules }` after movement and projectile impacts, before contact. Resolve toggles against stored energy, combine recharge/drain, disable at zero, then advance powered shield recharge. Contact consumes staged shield state. Commit only if phase remains Playing after contact/survival; terminal frames preserve previous battery/module state.
- [x] Migrate energy/overdrive tests to Modules without changing their behavioral assertions. Verify `cargo test --locked`, then commit the power model milestone.

## Task 2 — Shield and mobility integration

Files: `src/combat/lifecycle.rs`, `src/combat.rs`, `src/arena.rs`, `src/arena/flight.rs`, `src/combat/module_tests.rs`.

- [x] Add failing tests for same-frame shield activation, depletion before contact, blocked overlapping hits, powered-only recharge and frozen/reset state.
- [x] Route eligible contact through `PowerFrame.modules.block(&ModuleConfig)`; blocked contact grants existing invulnerability without hull loss or PlayerDamaged event.
- [x] Add failing flight tests comparing real velocity with boost on/off, vertical invariance and cap restoration. Add `FlightConfig.horizontal_acceleration_multiplier` default 1; scale X/Z thrust before speed taper. Player copies base config and scales that multiplier and max_horizontal_speed while mobility is active. Enemy configs remain unchanged. Movement consumes previously committed power state; restoration occurs next movement update.
- [x] Verify relevant tests and commit combat integration.

## Task 3 — Rocket launcher

Files: create `src/combat/rockets.rs`, `src/combat/rocket_tests.rs`; modify `src/combat/weapon.rs`. Root integrates declarations, schedule, reset and visuals separately.

Interfaces: `Modules::active(ModuleKind::Rocket)`, `ModuleConfig` with `rocket_interval: f64`, `rocket_damage: u32`, `rocket_radius: f32`, `rocket_speed: f32`, `rocket_lifetime: f32`, `rocket_range: f32`. `rockets::Rocket` marker on ordinary Projectile entities; `rockets::RocketLauncher` resource holding cooldown, default/reset; `rockets::fire` system. Existing projectile sweeps account for curved moving-enemy paths. Reuse them.

- [x] Write failing tests for splash once per target, 3D radius, misses/lifetime, stable automatic targeting, off/depletion stopping launch, in-flight continuity, cooldown toggling and cadence across frame rates.
- [x] Fire straight rockets using configured range/speed/lifetime and independent cooldown. No banked bursts; use normal weapon hitch protections. Basic overdrive never changes rocket cadence.
- [x] On first swept collision, compute impact position/time; apply one rocket_damage to direct target and enemies within inclusive rocket_radius at that time. Reuse existing Hit outcomes and kill accounting. Add `RocketExplosion { position, radius }` combat outcome for presentation (root integrates variant). Preserve ordinary projectile behavior.
- [x] Verify targeted tests, report changes and integration details, commit owned files after coordinating with root.

## Task 4 — Presentation and validation

Files: `src/energy/scene.rs`, `src/modules/scene.rs`, combat scene/feedback, README, playtests.

- [x] Show every slot's key/name/state/actual and configured drain. Show shield ready/recharge/paused state and per-slot activation rejection. Shared battery shows total drain and signed charging net, including -11/s for all four.
- [x] Add distinct rocket mesh/material and short capped impact-radius effect; reuse assets and clear on restart. Preserve 640x480 readability.
- [x] Add meaningful HUD/state/reset tests. Run `cargo fmt --check`, `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings`.
- [x] Run native `cargo dev` validation and inspect UI; compare maximum-output and conservation on same encounter, recording evidence and limitations.
- [x] Independent whole-branch review; resolve findings, update README/spec/playtest evidence, commit, push, open PR against main.


## Execution record

- Scope and plan committed as `1ace123`; gameplay/presentation milestone as `b27813b`.
- Power/shield/mobility review: no actionable findings.
- Whole-branch review of b27813b: ready for PR, no actionable findings.
- Added final visual-lifecycle regression and documented native/encounter evidence.
- Validation limitation: the shared comparison pilot rarely visits chargers; human
  tactical balance assessment is still needed. See `docs/playtests.md`.
