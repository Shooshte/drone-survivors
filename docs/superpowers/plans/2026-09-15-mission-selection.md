# Mission Selection Implementation Plan

> Execute task-by-task with test-first checks and review at milestones.

**Goal:** Deliver all 12 identical playable placeholder missions with campaign unlocks and free replay.

**Architecture:** Pure MissionId/Progress rules in src/mission/campaign.rs; mission.rs snapshots identity and routes selection; selection_scene.rs renders the catalog using current menu conventions.

**Tech Stack:** Rust, Bevy, existing Cargo/native macOS setup.

## Global constraints

All missions reuse the current 300-second encounter and rewards unchanged. Three acts: introduction → either branch order → finale → next introduction. Session-only campaign state. Existing direct-combat fixtures remain compatible. Minimum window 640×480.

## Task 1 — Campaign and attempt identity

- [x] Add failing tests in src/mission/campaign_tests.rs using the existing app/tick/launch harness. Prove distinct mission IDs in results, branch prerequisites, failure/replay/restart behavior and all-12 completion.
- [x] Run `cargo test --locked mission::campaign_tests` and inspect the missing-behavior failure.
- [x] Implement MissionId, Progress and mission metadata in src/mission/campaign.rs; replace the single-success boolean and add active/selected mission snapshots in src/mission.rs. Guard locked launches, record active identity once, preserve rewards and loadout behavior.
- [x] Run mission tests, repair affected legacy assertions and commit campaign rules.

## Task 2 — Selection UI and controls

- [x] Add routing tests for C, arrow navigation, locked mouse actions, Enter/Backspace, held/simultaneous input and paused selection. Add presentation checks for progress, prerequisites and mission identity.
- [x] Observe failures, then implement GamePhase::MissionSelect, MissionAction selection variants, and src/mission/selection_scene.rs. Keep hub briefing shortcut; add selection button. Update briefing/results/hub copy in scene.rs.
- [x] Include selection in explicit menu freeze matches in combat/waves.rs and upgrades/runtime.rs. Run all tests and commit UI integration.

## Task 3 — Native evidence and delivery

- [x] Add opt-in campaign validation using normal menu actions and explicitly synthetic terminal outcomes. Exercise all acts in both branch orders, replay/failure/restart, campaign completion, and capture selection, briefing, results and final hub at both supported sizes.
- [x] Run cargo fmt --check, cargo test --locked and cargo clippy --all-targets --locked -- -D warnings. Use cargo dev for native fixture and ordinary keyboard/mouse smoke checks; inspect screenshots and bounds.
- [x] Request independent code review, address actionable findings, and update README.md and docs/playtests.md with test results and limitations. Commit evidence.
- [ ] Push codex/dro-17-mission-selection, open PR against main, and update DRO-17 with delivery/evidence links.
