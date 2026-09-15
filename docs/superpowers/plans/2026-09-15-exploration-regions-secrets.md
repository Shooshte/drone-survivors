# Exploration, Regions, and Secrets Implementation Plan

> **For agentic workers:** Use subagent-driven-development for the bounded persistence task and independent review; integrate region runtime and presentation in this workspace.

**Goal:** Make exploration reward run XP, durable discoveries, and an upgrade forfeited on defeat/restart while preserving the placeholder arena.

**Architecture:** Region profiles supply economy and charging configuration at launch. Existing caches carry discovery identities; campaign secrets and finale routes persist through the existing atomic save flow. Mission lifecycle owns purchase loss, baseline application, and input.

**Tech Stack:** Rust, Bevy 0.19.1, serde JSON; no new dependencies.

## Global Constraints

- All acts retain the current placeholder arena layout.
- Secret discoveries unlock their campaign rewards immediately, including on attempts that later fail or restart.
- The purchased bonus survives successful missions but is lost on defeat or manual mission restart.
- The blueprint remains unlocked, allowing another purchase.
- Reserve battery costs 20 salvage and 1 component and adds 25 maximum battery capacity after permanent passives and before temporary modifiers.
- Each cache awards 30 run XP once per attempt. Normal progression and all 12 distinct victories remain available/required.
- Use the approved spec at `docs/superpowers/specs/2026-09-15-exploration-regions-secrets-design.md` for content values.

## Task 1: Durable secrets and upgrade lifecycle

Files: create `src/mission/secrets.rs`; modify `src/mission.rs`, `src/mission/campaign.rs`, `src/upgrades/runtime.rs`, `src/save.rs`, `src/save/snapshot.rs`; add focused mission/save tests.

Interfaces: `Campaign.secrets: Secrets`; `Secrets` exposes `blueprint: bool`, `reserve_battery: bool`, `discover_blueprint() -> bool`, `purchase(&mut Amounts) -> Result<(), &'static str>`, `lose_battery()`; `Progress::discover_route(act: usize) -> bool`, `routes() -> [bool; 3]`. Mission action `BuyReserveBattery` buys from Passives with key B.

- [ ] Write tests for insufficient/exact payment, duplicate purchase, blueprint idempotence, route access without completion, normal progression, and invalid active purchase without blueprint.
- [ ] Run focused tests to establish missing behavior.
- [ ] Implement secrets, routes, mission purchase/restart/death handling, baseline capacity bonus.
- [ ] Extend snapshot defaults for old saves, validate discovered-route reachability, allow immediate durable campaign writes during play/choices, correctly resume clocks after save retry.
- [ ] Test immediate save/reload, death/restart loss, repurchase, success retention, old saves and write-error recovery.
- [ ] Commit only task-owned files after focused tests pass.

Core assertion examples:
```rust
assert!(!campaign.progress.unlocked(MissionId::ALL[3]));
assert!(campaign.progress.discover_route(0));
assert!(campaign.progress.unlocked(MissionId::ALL[3]));
assert_eq!(campaign.progress.count(), 0);
assert!(!campaign.progress.discover_route(0));
```

## Task 2: Shared region content and cache discoveries

Files: create `src/world/regions.rs`; modify `src/world.rs`, `src/world/layout.rs`, `src/economy/runtime.rs`, `src/energy.rs`; add `src/economy/discovery_tests.rs`.

Interfaces: `RegionProfile::for_mission(MissionId)` returns named data for drop chance, component amount, charger capacity, and a resource-summary method. Reusable content pieces expose existing cache/charger sites without moving any geometry. `DiscoveryNotice` stores the latest collection feedback.

- [ ] Write runtime tests that launch each act and inspect actual drops, cache awards, and charger reserve values.
- [ ] Run tests and confirm missing profile/discovery behavior.
- [ ] Configure profiles during Baseline before charger/economy Reset; preserve legacy direct-combat fixtures.
- [ ] Tag existing caches with stable indices. Credit XP and campaign unlocks exactly once on successful collection, with swept path + clear line of sight and terminal/reset/pause guards.
- [ ] Test fast crossings, walls, repeated contact, reset, fatal-frame precedence, region switching and fixed placement.
- [ ] Commit after focused tests pass.

Collection contract:
```rust
if resources.collected.try_credit(pickup.amount).is_ok() {
    run.award(30);
    // Index 0 discovers blueprint; index 1 discovers current act finale route.
    commands.entity(entity).despawn();
}
```

## Task 3: Presentation and native validation

Files: `src/passives/scene.rs`, `src/mission/scene.rs`, `src/mission/selection_scene.rs`, `src/economy/scene.rs`; new native fixture alongside `src/mission/objective_validation.rs`, registered through `src/main.rs` and combat validation configuration.

- [ ] Show region summaries from launch data, discovered route status, and battery purchase/active/loss wording.
- [ ] Add compact battery purchase button to upgrades and a brief discovery notice in the resource HUD.
- [ ] Validate text/layout with native runs at 1120x720 and 640x480; inspect captured images.
- [ ] Exercise physical navigation to caches using cargo dev; disclose synthetic fixture inputs separately.
- [ ] Record checks and limitations in README and `docs/playtests.md` plus a dedicated evidence file.
- [ ] Commit completed presentation and evidence.

## Task 4: Review and delivery

- [ ] Run `cargo fmt --check`, `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings`.
- [ ] Dispatch an independent whole-branch review; fix and verify actionable findings.
- [ ] Push `codex/dro-20-exploration` and open PR against `main`, linking DRO-20 and evidence.
- [ ] Update Linear to In Review with the PR and validation summary.

## Progress

- Baseline: 361 passed, 4 ignored; branch starts at ef88578.
- Approved scope recorded in Linear and issue set In Progress before implementation.
