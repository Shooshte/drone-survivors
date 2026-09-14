# Module shop implementation plan

> Execute tasks in this session with test-first development and independent review. Approved behavior: ../specs/2026-09-14-module-shop-design.md.

**Goal:** Buy the four existing modules and launch with a freely edited, owned loadout.

**Architecture:** Campaign owns inventory and planned equipment. Mission transitions snapshot validated equipment per attempt. The shop and briefing share previews calculated from permanent tuning; combat resets preserve the snapshot.

**Tech Stack:** Existing Rust/Bevy 0.19.1; no additional dependencies.

## Global constraints

New campaigns own no modules and start with four empty slots. Prices (salvage/components): Overdrive 10/0, Shield 15/0, Mobility 15/0, Rockets 25/1. Session-only persistence; empty launches are valid. Assignment/removal costs nothing and duplicate types are forbidden. Existing combat fixtures retain their default loadout. Readable at 1120×720 and 640×480.

## Task 1: Inventory and purchase rules

Files: create `src/modules/shop.rs` and `src/modules/shop/tests.rs`; register in `src/modules.rs`.

Interfaces: `ModuleInventory::default()`, `owns(ModuleKind) -> bool`, `purchase(ModuleKind, &mut Amounts) -> Result<(), ShopError>`, `assign(usize, Option<ModuleKind>) -> Result<(), ShopError>`, `loadout() -> &Loadout`, `validate() -> Result<(), ShopError>`, `price(ModuleKind) -> Amounts`. Private ownership and loadout protect invariants. Add ordering to ModuleKind for deterministic UI actions.

- [x] Write tests for exact/insufficient currency, duplicate purchase, four empty slots, unowned rejection, invalid slot, moves and replacements preserving ownership/balance. Establish failing behavior before implementation.
- [x] Implement catalog-backed transactions using Amounts::try_spend, no automatic equip; free moves clear source then replace destination. Validate ownership and unique slots through Loadout::new.
- [x] Run focused tests, self-review and commit inventory milestone.

## Task 2: Mission integration and preview

Files: `src/mission.rs`, `src/game.rs`, `src/energy.rs`, `src/modules/shop_input.rs`, `src/mission/module_tests.rs`, `src/modules/shop_preview.rs`.

- [x] Add integration tests: shop only from hub, inputs require release, one action per frame, insufficient funds, valid empty launch, purchased arrangement exactly matches runtime and survives R/results/replay. Run tests red.
- [x] Add Campaign inventory and MissionSession active loadout snapshot; use snapshot on energy reset. Route shop actions through existing MissionAction input. Scope inputs to shop, suppress launch on invalid ownership; restart reuses snapshot.
- [x] Compute preview from pristine tuning plus campaign passives, with tests excluding previous temporary upgrades and including battery/charge passives.
- [x] Run focused and full tests, commit integration milestone.

## Task 3: Shop, briefing and native validation

Files: `src/modules/shop_scene.rs`, `src/mission/scene.rs`, native validation registration and fixture, README.md and docs/playtests.md.

- [x] Add scene tests for inventory/price/selection/loadout feedback and actual briefing equipment. Implement catalog, selected detail panel, purchase, slot assignment/removal, bank and potential energy summary in existing palette.
- [x] Add a deterministic native fixture using real menu actions and controlled funds. Verify keyboard/mouse purchases, free rearrangement, insufficient funds, empty slots, launch/restart and both target window sizes using cargo dev. Inspect screenshots and repair clipping if observed.
- [x] Run cargo fmt --check, cargo test --locked and cargo clippy --all-targets --locked -- -D warnings. Record commands/results and provisional balance limits in docs/playtests.md.
- [x] Review branch, fix actionable findings and record evidence. Delivery: push branch, open PR against main and update DRO-16 after this evidence commit.

## Execution record

Inventory: ee7c1fc. Mission integration: b2d2a57. UI/native fixture: 0278ab6. Independent reviews approved; final copy clarification included with playtest evidence. Final checks: 323 tests passed, 4 ignored; formatting and strict Clippy passed; both native window sizes passed. See docs/playtests/dro-16-module-shop.md.
