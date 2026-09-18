# Enemy variants implementation plan

**Goal:** Deliver DRO-35 in one PR against main.
**Architecture:** Existing wave and rotor-flight systems with an optional arena
roster, per-kind tuning and an attached rammer state machine; cached visual cues.
**Tech stack:** Rust, Bevy 0.19.1, native macOS, existing Cargo tooling.

## Constraints

- Worktree `.worktrees/dro-35-enemy-variants`, branch `codex/dro-35-enemy-variants`.
- New types occur only in catalog scenarios; preserve campaign/save behavior.
- Two separate impacts, physically separated retreat, full visible warning.
- Agent evidence cannot complete the human content gate.

## 1. Spawn and identify the new enemies

- [x] Add a failing catalog regression for the new selectable scenarios.
- [x] Extend EnemyKind; put per-kind tuning and Rammer state in
  `src/combat/variants.rs`, register it from `src/combat.rs`.
- [x] Add an optional arena spawn roster, capture its kind in SpawnWarning and
  use a typed spawn helper. Keep existing spawn_enemy as the ordinary-chaser
  entry for legacy fixtures. Extend catalog scenario metadata and instructions.
- [x] Test typed activation and isolation; commit.

## 2. Physical attacks, contact and feedback

- [x] Write tests for rammer phases, impact budgets, shield/invulnerability,
  reset/pause, swept crossings and terrain before implementing those rules.
- [x] Feed variant targets/profiles into `enemies::chase`; retain rotor physics.
- [x] Extend contact_damage with optional Rammer state and segment-based new-type
  contact; preserve ordinary chaser contact. Separate accepted impact accounting
  from kill rewards. Exercise normal projectile and economy event consumers.
- [x] Cache variant meshes/materials, rings and impact indicators in a dedicated
  scene module. Make feedback restore the material corresponding to type/state.
- [x] Run focused frame-rate/terrain/kill tests; commit.

## 3. Verify and deliver

- [x] Extend explicit native fixture for both new scenarios and rammer cues;
  inspect screenshots at 640x480 and 1120x720. Record synthetic setup precisely.
- [x] Run formatting, full tests and strict Clippy; independent review and fixes.
- [x] Record results in docs/playtests.md and a DRO-35 report; update README.
- [x] Commit, push, open PR against main and move DRO-35 to In Review with links.

## Completion notes — September 18, 2026

Resumed the existing approved worktree and partial implementation. Initial suite
passed 399 tests; completed native fixture and independent review. Review fixes
were reproduced red/green with extra recovery tests; final suite passed 403 tests
with five existing probes ignored. See `docs/playtests/dro-35-enemy-variants.md`.
