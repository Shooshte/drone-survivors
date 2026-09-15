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

- [ ] Add a failing catalog regression for the new selectable scenarios.
- [ ] Extend EnemyKind; put per-kind tuning and Rammer state in
  `src/combat/variants.rs`, register it from `src/combat.rs`.
- [ ] Add an optional arena spawn roster, capture its kind in SpawnWarning and
  use a typed spawn helper. Keep existing spawn_enemy as the ordinary-chaser
  entry for legacy fixtures. Extend catalog scenario metadata and instructions.
- [ ] Test typed activation and isolation; commit.

## 2. Physical attacks, contact and feedback

- [ ] Write tests for rammer phases, impact budgets, shield/invulnerability,
  reset/pause, swept crossings and terrain before implementing those rules.
- [ ] Feed variant targets/profiles into `enemies::chase`; retain rotor physics.
- [ ] Extend contact_damage with optional Rammer state and segment-based new-type
  contact; preserve ordinary chaser contact. Separate accepted impact accounting
  from kill rewards. Exercise normal projectile and economy event consumers.
- [ ] Cache variant meshes/materials, rings and impact indicators in a dedicated
  scene module. Make feedback restore the material corresponding to type/state.
- [ ] Run focused frame-rate/terrain/kill tests; commit.

## 3. Verify and deliver

- [ ] Extend explicit native fixture for both new scenarios and rammer cues;
  inspect screenshots at 640x480 and 1120x720. Record synthetic setup precisely.
- [ ] Run formatting, full tests and strict Clippy; independent review and fixes.
- [ ] Record results in docs/playtests.md and a DRO-35 report; update README.
- [ ] Commit, push, open PR against main and move DRO-35 to In Review with links.
