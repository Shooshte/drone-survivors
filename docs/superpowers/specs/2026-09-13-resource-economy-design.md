# DRO-14 resource economy

Approved through the DRO-14 refinement conversation and Linear issue. Implementation starts from main 975fd8f (DRO-13 merged).

## Rules

- New session campaign: 0 salvage, 0 components; mission entry and basic equipment are free.
- Chaser: 25% chance of 1 salvage per confirmed death; chance and amount are configured by enemy type. Separate repeatable loot RNG, unaffected by waves/XP RNG.
- Salvage rests on ground or supporting low cover, attracts within 100 world units in 3D with line of sight, no interaction key. It persists until collected/end. Uncollected drops award nothing.
- Three fixed component caches, 1 component each, collected within 50 units in 3D and line of sight. Reset each attempt. Place near the north/south spawn perimeter and electrical passage; reachable by the Scout. These are testing placeholders.
- Success banks collected amounts + 10 salvage/1 component. Failure banks floor(collected / 4) separately, no bonus. Replayed successes also pay.
- Settle once during DRO-13 completion; R aborts with no payout, clears collection/drops, restores caches. Collection is after combat outcome resolution and before choices; terminal outcomes and reset take precedence.
- Banks last for the app session. Saving, shop/passive screens and handcrafted missions are downstream.

## Architecture

Pure `economy.rs` holds integer amounts, checked atomic credit/debit, drop rules and reward calculation. `economy/runtime.rs` owns attempt pickups, deterministic death rolls and collection. It consumes the already deduplicated combat hit facts, additionally guards death identities, and uses explicit mission reset/cleanup boundaries. The mission completion system adds a receipt to its stable result and banks the campaign wallet once. A checked-arithmetic failure leaves the wallet unchanged and records an explicit settlement error rather than panicking or partially crediting.

`economy/scene.rs` shares cached visual handles, attaches visible salvage/caches and a small unbanked HUD row. Existing mission menus show banked balances and compact two-currency result breakdowns. Normal mission play installs economy; legacy direct-combat fixtures stay unchanged. Extend the opt-in native mission fixture for economy evidence, clearly disclosing synthetic setup.

## Validation

Cover arithmetic/rounding, exact and insufficient spending, failed credit rollback; controlled drop/no-drop, duplicate deaths including multikills/hazards; geometry, attraction/pause/no expiry; cache resets, success/failure/replay, terminal/reset-frame precedence. Run formatting, full locked tests, strict all-target Clippy and native UI/gameplay checks at 1120x720 and 640x480. Preserve the independent original checkout.
