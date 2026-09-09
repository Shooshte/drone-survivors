# Experience and Temporary Choices Implementation Plan

> **For agentic workers:** Use subagent-driven-development for isolated rules/UI work and review. Root integrates runtime and verifies the whole feature.

**Goal:** Implement DRO-10, including six trade-offs and explicit Skip, in the existing arena.

**Architecture:** Typed upgrade rules own XP, offers, and modifiers; runtime bridges combat outcomes to these rules and applies derived stats. A modal and pickup/HUD presentation consume the same state. Pause Bevy virtual time while choosing so absolute gameplay deadlines remain valid.

**Tech Stack:** Rust, Bevy 0.19.1; no new dependencies.

## Global Constraints

Read `docs/superpowers/specs/2026-09-09-experience-design.md` for exact catalog, XP values, and acceptance criteria. Heavy armor gives +50 hull / −25% all translational acceleration. Heavy rounds doubles basic damage / −10% all translational acceleration with no firing slowdown or energy cost. Effects reset per run. Skip consumes one earned choice, applies no effect, and advances the queue or resumes. No permanent progression. Preserve gameplay at an empty battery.

## Shared interfaces

`src/upgrades.rs` owns:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UpgradeKind { Interceptor, AgileFrame, HeavyArmor, HeavyRounds, RapidShield, WideAreaRockets }
// name(), benefit(), drawback() -> &'static str; ALL: [Self; 6]
#[derive(Resource)]
pub(crate) struct UpgradeRun {
    pub level: u32, pub xp: u32, pub pending: u32,
    pub selected: Vec<UpgradeKind>, pub offer: Vec<UpgradeKind>,
    pub exhausted: bool,
    // private reproducible RNG state
}
// Default starts level1; award(amount:u32); threshold()->u32;
// prepare_offer(&mut self, loadout:&Loadout); resolve(&mut self,index:Option<usize>)->bool;
// resolve(None) skips, rejects invalid indices without consuming; prepare_offer filters prerequisites,
// clears pending and sets exhausted if pool empty, never rerolls a nonempty offer.
#[derive(Component, Clone, Copy)]
pub(crate) enum ChoiceAction { Pick(usize), Skip }
```
`UpgradeModifiers::from_selected(&[UpgradeKind])` derives public fields: `horizontal_acceleration:f32`, `speed:f32`, `handling:f32`, `acceleration:f32`, `hull_delta:i32`, `capacity:f64`, `shot_damage:u32`, `shield_recharge:f64`, `shield_drain:f64`, `rocket_radius:f32`, `rocket_interval:f64`. Multiplicative identity1, hull_delta0, shot_damage multiplier1. Hull changes are +50 and −20 baseline units.

Runtime file exports `UpgradePlugin`, `ExplorationPickup { position:Vec3, radius:f32, collected:bool }` resource, and installs reset / input / XP accounting in explicit ordered gameplay sets. `GamePhase::Choosing` represents the modal. `scene::UpgradeScenePlugin` renders `UpgradeRun` and pickup, with buttons carrying `ChoiceAction`; runtime consumes their interactions.

## Task 1 — Rules and tests

Files: `src/upgrades.rs`, `src/upgrades/tests.rs`; declare `mod upgrades;` in main.

- [ ] Add failing tests for threshold carry, multi-level award, stable eligible offers, selection/skip, depleted pools, deterministic sampling, and composition.
- [ ] Implement the shared interface with six typed definitions and reproducible sampling without adding dependencies.
- [ ] Check representative rule assertions:
```rust
let mut run = UpgradeRun::default();
run.award(140);
assert_eq!((run.level, run.xp, run.pending), (3, 15, 2));
run.prepare_offer(&Loadout::default());
assert_eq!(run.offer.len(), 3);
assert!(run.resolve(None));
assert!(run.selected.is_empty());
assert_eq!(run.pending, 1);
```
- [ ] Run `cargo test --locked upgrades` (red then green) and review the rules diff.
- [ ] Commit rules once verified.

## Task 2 — Runtime and combat integration

Files: `src/upgrades/runtime.rs`, `src/upgrades/runtime_tests.rs`, `src/game.rs`, `src/arena.rs`, `src/arena/flight.rs`, `src/combat.rs`, `src/combat/{weapon,rockets,scene,waves,validation}.rs`, `src/energy.rs`.

- [ ] Write failing integration checks against real arena/combat systems for exact-once XP, choice pause, skip, restart, and both acceleration penalties.
- [ ] Add ordered choice-input and progression sets. Capture pristine baseline resources at startup, derive modifiers on selection, and restore baseline before restart systems run.
- [ ] Snapshot basic/rocket damage and rocket radius at launch. Preserve cooldown/shield progress when applying choices.
- [ ] Pause virtual time on opening; suppress resumed gameplay and toggle leakage on the resolving frame; release-gate input between queued offers. Only terminal states cancel warnings.
- [ ] Expose confirmed kill count safely for run-local XP; test multiple kills and restart boundaries.
- [ ] Add one pickup at (0,90,-180) radius30, then award30 XP once when drone center enters inclusive 3D radius.
- [ ] Verify long pauses with TimePlugin, collision/terminal ordering, module state, empty pools, and exact baseline reset using `cargo test --locked`.
- [ ] Commit integrated gameplay after checks.

## Task 3 — Choice UI and presentation

File: `src/upgrades/scene.rs` plus main plugin registration.

- [ ] Build a compact dark modal: title, run duration label, up to three cards with separate benefit/drawback text, explicit Skip/Backspace button, queue count, restart hint.
- [ ] Reuse existing UI style and Bevy nodes. At 640x480 use stacked compact cards; all cards and Skip must remain visible.
- [ ] HUD adds level/progress and acquired names without overlapping existing top HUD or bottom module rows. Show exhausted notice.
- [ ] Show a visible 3D pickup radius and center marker at the runtime pickup resource position; collected hides it, reset restores it.
- [ ] Verify correct visibility/card text/skip action with focused presentation checks and native screenshots.
- [ ] Commit verified UI with integration.

## Task 4 — Validation, documentation, review, PR

Files: `src/combat/validation.rs`, `README.md`, `docs/playtests.md` and relevant tests.

- [ ] Keep existing validation modes functional with choice pauses. Add deterministic build selection in explicit validation runs to compare mobile and armored builds without altering normal gameplay.
- [ ] Run native `cargo dev` scenarios, inspect 640x480 UI and record real findings/timing/limitations.
- [ ] Run `cargo fmt --check`, `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings`.
- [ ] Independently review the whole branch, address findings, and rerun affected checks.
- [ ] Commit documentation and validation evidence, push `codex/dro-10-experience`, create PR targeting main. Keep the worktree for review.
