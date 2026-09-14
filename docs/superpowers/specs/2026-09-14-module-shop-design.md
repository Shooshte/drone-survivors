# DRO-16 module shop and loadout

Agreed in conversation September 14, 2026. Start from main bb9c8bb, with DRO-9, DRO-14 and DRO-15 merged. Prices are provisional and adjustable during balance playtesting.

## Rules

New campaigns own no modules and have four empty slots. The basic weapon, flight and mission entry are free; an empty loadout is valid. All four existing modules are available to purchase immediately, without mission prerequisites. Purchase permanently unlocks one module type for the application session. No new module effects, refunds, resale, tiers or disk persistence (DRO-18).

| Module | Salvage | Components |
| --- | ---: | ---: |
| Overdrive | 10 | 0 |
| Shield | 15 | 0 |
| Mobility | 15 | 0 |
| Rockets | 25 | 1 |

Purchases debit banked resources atomically. Already owned or unaffordable purchases change neither bank nor inventory. Purchasing does not auto-equip. Assignment, movement and removal are free. Any owned type can occupy any slot, at most once; moving an equipped module clears its old slot and replaces the destination, returning its prior module to inventory. Removing a module preserves ownership.

The paused hub gains a module shop/loadout screen. Select a catalog entry with mouse or Up/Down, buy with B or its buy button, assign to slot 1–4, and remove with that slot's remove button or Shift+1–4. Backspace returns to hub. Show all owned/unowned states, price, affordability, effect, drain and tradeoffs before purchase/launch. Guard held inputs and multiple simultaneous actions with the existing mission input arm/release behavior.

Briefing displays the actual four slots and an energy summary derived from pristine tuning plus purchased passives, excluding stale temporary upgrades: battery capacity, total drain with all equipped modules ON, charger supply and net rate while supplied. Label this as potential drain: all modules start OFF and empty slots drain zero. Warn about reserve depletion; charger supply is conditional. No power-budget restriction on launch. Validate ownership and uniqueness at launch. Snapshot the displayed loadout for the attempt; R restarts reuse it, while fresh launches use the hub selection. Purchases/loadout survive results and replay within the session. Existing direct-combat fixtures retain their four-module setup.

## Architecture and verification

`modules/shop.rs` owns catalog prices, session inventory, atomic purchase and loadout edits; Campaign owns that state. `mission.rs` routes shop inputs and captures launch equipment; `energy.rs` resets from the active attempt snapshot. `modules/shop_scene.rs` presents the shop and uses the existing menu palette. A shared preview helper derives launch tuning from the pristine baseline and passives. Menu integration remains readable at 1120×720 and 640×480.

Test affordability for either currency, exact funds, repeat purchases, ownership rejection, every slot, moves/replacements, empty launch, launch snapshot/restart/replay, paused/held/multiple input and passive-aware energy previews. Run cargo fmt --check, cargo test --locked and cargo clippy --all-targets --locked -- -D warnings. Use cargo dev for native keyboard/mouse and layout checks; record evidence in docs/playtests.md. Commit milestones, push codex/dro-16-module-shop and open PR against main.
