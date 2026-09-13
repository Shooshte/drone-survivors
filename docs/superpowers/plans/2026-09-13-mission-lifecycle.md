# Mission Lifecycle Implementation Plan

> **For agentic workers:** Use subagent-driven-development or executing-plans to implement this plan task-by-task. Track completion below.

**Goal:** Implement DRO-13's approved repeatable hub/briefing/combat/results loop.

**Architecture:** Retain the existing combat plugins and their direct-combat default for validation fixtures. Add an optional MissionPlugin for normal startup that owns transitions and session-only campaign history, and a MissionScenePlugin for menus. Coordinate resets explicitly across gameplay systems; never synthesize keyboard events. Reuse terminal GamePhase values for success/failure results, adding Hub and Briefing.

**Tech Stack:** Rust, Bevy 0.19.1; native macOS; no new dependencies.

## Global constraints

- Binding scope: `docs/superpowers/specs/2026-09-13-mission-lifecycle-design.md`, copied from the approved Linear ticket.
- 300 active-second existing survival encounter; unchanged combat tuning and upgrade rules.
- Placeholder narrative text allowed; Scout and current four modules fixed before launch; combat toggles work.
- Session-only campaign history; no rewards, save files, shops, unlock graph or additional objectives.
- Keyboard and mouse at 1120×720 and 640×480; Enter primary action, Backspace briefing back, Escape quit, R combat/choice restart only.
- Fresh input per transition; terminal completion once per attempt; no advancing/catching up gameplay outside active combat.
- Share `/Users/shooshte/projects/drone-survivors/target` for Cargo build artifacts; all source changes stay in this worktree.

## Task 1: Lifecycle and explicit reset integration

Files: create `src/mission.rs`, `src/mission/tests.rs`; modify `src/game.rs`, arena/combat/energy/upgrades reset and phase integration where required. Core worker owns these files. Scene/main are Task 2 ownership.

Public crate interfaces for the scene:

```rust
// In game.rs: existing phases plus Hub, Briefing.
// In mission.rs:
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MissionAction { Briefing, Launch, Hub }
pub(crate) struct MissionPlugin;
// resource types expose:
// Campaign { pub history: Vec<MissionResult>, pub mission_succeeded: bool }
// MissionSession { pub result: Option<MissionResult>, ...private attempt/input state }
// MissionResult { pub attempt: u64, pub succeeded: bool, pub elapsed: f64, pub kills: u32 }
```

- [x] Add behavioral tests through real gameplay plugins: hub freeze and R ignored; launch clears dirty state; success/failure and completion deduplication; restart from a paused choice; campaign retention; elapsed-time and outcome precedence; held/clicked input isolation.
- [x] Run focused tests and capture expected failure before implementing logic. Existing direct-combat tests retain their default startup and restart contract.
- [x] Implement MissionPlugin transitions before reset and completion after progression, with immutable result snapshots, unique attempt ids, guarded input and explicit reset integration. Restore upgrade baseline before dependent resets. Freeze clock with no wall-time catch-up; reset frame must not advance combat or apply movement/module input.
- [x] Exercise full dirty-state reset and entity cleanup through real integration fixtures, using private combat tests as needed. Run `CARGO_TARGET_DIR=/Users/shooshte/projects/drone-survivors/target cargo test --locked`.
- [x] Commit the integrated lifecycle and presentation after focused and full checks (5ef593e); a combined source commit keeps every revision buildable.

## Task 2: Mission presentation and normal startup

Files: create `src/mission/scene.rs`; modify `src/main.rs`, README. Root worker owns these files.

Consumes Task 1 interfaces above. MissionScenePlugin installed only when normal MissionPlugin is installed. Scene emits no synthetic input: clickable buttons carry MissionAction, consumed by core input handling.

- [x] Build reusable menu entities for Hub, Briefing, Results; change display by GamePhase so no UI accumulates. Read Campaign for completed attempt count/prior success and MissionSession.result for results.
- [x] Use existing Bevy typography and blue/cyan arena palette; opaque backdrop hides frozen arena HUD. A compact operations panel has clear heading, readable objective/loadout, a prominent action and separate keyboard help. Keep the menu under 640×480 available area; no unnecessary graphics/assets.
- [x] Add meaningful scene tests for screen visibility and interaction controls across transitions, stable results, and repeated-update entity counts. Verify screenshot rendering rather than duplicating static text assertions.
- [x] Register mission module and install plugins only for normal startup. Update title and README controls/flow. Preserve validation CLI parsing and direct-combat installation.
- [x] Run focused tests, launch natively, inspect both window sizes and click/keyboard routes. Presentation is included in the integrated source commit 5ef593e.

## Task 3: Integration review, native verification and PR

Files: tests as needed, `docs/playtests.md`, this checklist.

- [x] Review each completed unit for spec and code quality, then resolve findings with covering tests.
- [x] Demonstrate normal hub → briefing → launch, restart from choice, death/results → hub → launch, and success/results. Record whether native terminal outcomes were natural or fixture-driven. Keep synthetic state confined to tests/explicit validation fixtures.
- [x] Run `cargo fmt --check`, `cargo test --locked`, and `cargo clippy --all-targets --locked -- -D warnings` with shared CARGO_TARGET_DIR. Record exact results and any limitations in playtests.
- [x] Obtain independent whole-branch review against origin/main; fix material findings and rerun affected checks.
- [x] Commit evidence/docs, push branch, open PR targeting main with concise problem/behavior/validation summary and DRO-13 link. Retain branch/worktree for review.
