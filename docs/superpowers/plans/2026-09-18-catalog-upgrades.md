# Catalog Upgrades Implementation Plan

> **For agentic workers:** Use subagent-driven-development for the bounded upgrade rules/runtime task and requesting-code-review for reviews.

**Goal:** Deliver DRO-40's approved six additional upgrades in the isolated arena.

**Architecture:** Extend the existing baseline modifier pipeline. A default-safe UpgradePool resource enables catalog offers and optionally queues up to four selected preview cards through normal choices. A second arena setup page configures previews without introducing another effect pipeline.

**Tech Stack:** Rust, Bevy 0.19.1, existing Rust tests/native fixtures.

## Global Constraints

- Four opportunities per run, including preview Pick/Skip; campaign offers retain exactly the original six.
- All additions are catalog-only, with no campaign save-format changes.
- Efficient coils: drain x0.75 for all six modules; activation threshold x2; at least one module equipped.
- Reserve battery: capacity x1.5, maximum horizontal speed x0.8; no free battery refill.
- Long-range rounds: basic targeting range x1.5, basic interval x1.25.
- Hot overdrive: powered firing multiplier x1.5, Overdrive drain x1.5; requires Overdrive.
- Rapid repair: repair rate x2, Repair drain x1.5; requires Repair.
- Wide repulsor: radius x1.5, interval x1.5; requires Repulsor.
- Effects derive once from stable baselines and compose; cooldown fractions survive choice; boundaries reset effects.
- Every card names benefit and drawback; evidence is agent validation, not human acceptance.

### Task 1: Upgrade rules and runtime

Files: src/upgrades.rs, src/upgrades/runtime.rs, focused new upgrade rule/runtime tests, existing combat upgrade test module if needed.

Interfaces: expose UpgradeKind::CATALOG (12), UpgradeKind::eligible(&Loadout), and UpgradePool resource with `catalog: bool` and `preview: Vec<UpgradeKind>` (default false/empty). UpgradePlugin initializes it only if absent. Arena installs true/empty and controls preview. Existing prepare_offer callers retain campaign defaults. At run reset, runtime sanitizes preview against current loadout (distinct, eligible, max four), grants cumulative XP for its count, and routes each through a one-card normal modal. Pending preview cards are consumed when offered so Skip cannot repeat them; any remaining earned choices use normal catalog sampling.

- [x] Write failing tests for pool isolation, new prerequisites, preview skips/budget and each modifier's composition/idempotence.
- [x] Implement six definitions with benefit/drawback copy, catalog-only sampling and preview queue; preserve `UpgradeKind::ALL` as campaign six.
- [x] Write failing runtime tests using production plugins for effective configs, battery clamp, activation behavior, paid repair, pulse cooldown fractions and reset.
- [x] Extend baseline application for new stats, scaling all drains and retiming the Repulsor cooldown without free pulses. Keep original behavior unchanged.
- [x] Run focused tests and commit the gameplay increment. Report exact tests and red/green evidence.

### Task 2: Arena setup and native evidence

Files: src/combat/catalog.rs, src/combat/catalog/scene.rs, new src/combat/catalog/upgrade_validation.rs, src/combat/catalog_tests.rs or focused child.

Interfaces: use Task 1's UpgradePool; write preview only while selecting. U toggles page, 1–4 cycle modules or preview cards according to page. Remove ineligible previews after module changes.

- [x] Write failing selector integration tests proving eligible distinct selection, keyboard/click parity, preview launch/skip budget and restart/return reset.
- [x] Add the focused preview page with four card rows, benefit/drawback copy, count and synthetic-XP explanation; preserve module/scenario page behavior and viewport fit.
- [x] Add opt-in DRONE_UPGRADE_SMOKE native fixture; exercise all twelve cards, combinations, four-limit, cooldown/power and reset through normal controls. Label synthetic XP and controls.
- [x] Run targeted integration tests and native fixtures at both supported window sizes, inspect screenshots/text bounds, then commit.

### Task 3: Review and delivery

Files: README.md, docs/playtests.md, docs/playtests/dro-40-catalog-upgrades.md, selected screenshots, this plan.

- [ ] Review gameplay task and final branch independently; fix actionable findings and rerun affected checks.
- [x] Run `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings`.
- [ ] Record exact evidence, provisional tuning and known limits. Commit coherent documentation.
- [ ] Push codex/dro-40-catalog-upgrades; open PR against main with Linear link and evidence.

## Progress

Baseline on origin/main 2e578e8: 478 passed, 5 ignored, zero failures. Worktree isolated at .worktrees/dro-40-catalog-upgrades.

Task 1: implemented in 482da18; 18 rule tests and 26 production runtime tests passed. Independent review pending.
Task 2: implemented in 0c371b9; four new selector integration tests pass. Native all-card fixtures pass at both sizes; module regression passes all14 scenarios after header-height correction.
Final automated verification: 494 passed / 5 ignored, formatting and strict Clippy clean. Native screenshots and logs recorded in docs/playtests/dro-40-catalog-upgrades.md.
