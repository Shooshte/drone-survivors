# Reusable Mission Objectives Implementation Plan

> Execute task-by-task using the executing-plans workflow, with a separate code review before delivery.

**Goal:** Make reconnaissance and cargo extraction playable in campaign missions 02 and 03.

**Architecture:** MissionPlugin installs an attempt-local ObjectiveRun and configurable locations. Combat keeps mortality and survival timing; a collection-stage system resolves visits/extraction before upgrades and the common completion transaction. Scene presentation reads the same objective definition.

**Tech Stack:** Rust, Bevy 0.19.1, existing Cargo/native validation harness.

## Global constraints

Follow the approved objectives design. Preserve existing rewards, saves, mission IDs,
unlock graph and direct combat fixtures. No additional dependencies. Work only in
`.worktrees/dro-19-objectives` on `codex/dro-19-objectives`.

## Task 1 — Objective rules and lifecycle

Files: create `src/mission/objectives.rs`, `src/mission/objective_tests.rs`;
modify `src/mission.rs`, `src/combat/waves.rs`.

- [ ] Add ECS regression: launch mission 02 with mission 01 completed, advance elapsed to 300, tick and assert phase Playing and no result. Run filtered test and observe failure.
- [ ] Define ObjectiveKind::{Survival,Reconnaissance,Extraction}, MissionId::objective(), ObjectiveRun { kind, visited: [bool;3] }, ObjectiveConfig { sites, extraction, visit_radius, extraction_radius }.
- [ ] Reset from active mission at GameplaySet::Reset. In Collection, gate on Playing and !reset, mark each distinct visible nearby point, and succeed only with all points and proximity to extraction. Finish before Progression.
- [ ] Gate survival victory and elapsed clamp by objective kind, defaulting to survival when resource absent.
- [ ] Test both kinds: early extraction, one visit repeated, third point still needs extraction, death wins, pause/restart, selected-vs-active identity, reward/save roundtrip. Run filtered tests then full suite.
- [ ] Commit rules and tests as one coherent milestone.

## Task 2 — Playable presentation and native fixture

Files: create `src/mission/objective_scene.rs`, `src/mission/objective_validation.rs`;
modify mission scene/selection, combat HUD, validation parser/campaign fixture and README.

- [ ] Add briefings and HUD tests expecting objective-specific instructions/progress; run and observe missing behavior.
- [ ] Render reusable markers at configured sites and extraction, hide completed cargo, color completed scans, show locked/ready extraction. Reuse assets across attempts.
- [ ] Show current progress and nearest remaining site's bearing/distance/height in HUD; show extraction guidance when complete. Distinguish elapsed from survival countdown.
- [ ] Add opt-in `--validate objectives`: use real menus, synthetic position changes and fatal hull to exercise both objective kinds, restart and success; assert results and capture screens. Adapt `--validate campaign` synthetic victories to objective kinds.
- [ ] Validate native UI/text bounds at both supported sizes and inspect screenshots. Exercise physical navigation using the native app.
- [ ] Commit presentation/fixture after relevant tests pass.

## Task 3 — Verification, review and delivery

- [ ] Run `cargo fmt --check`, `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings`.
- [ ] Review changes independently; resolve findings with regression tests.
- [ ] Record native results/limits in `docs/playtests.md` and dedicated evidence document.
- [ ] Commit evidence, push `codex/dro-19-objectives`, open PR against main, and update DRO-19 with delivery and validation.
