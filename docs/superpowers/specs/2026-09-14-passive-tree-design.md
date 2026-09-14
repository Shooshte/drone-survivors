# DRO-15 permanent passive tree

Approved in the DRO-15 refinement conversation on September 14, 2026. Base: origin/main 41b1d10, including merged DRO-14.

## Agreed behavior

Nine distinct nodes, five ranks each. Each rank adds a fixed percentage of the original base value; ranks do not compound. Branches are independent and nonexclusive. Rank 1 of a node unlocks the next node in that branch; subsequent ranks are optional. No refunds/respec in this chunk.

| Branch | Ordered node | Per rank | Maximum |
| --- | --- | --- | --- |
| Offense | Projectile damage | +10% basic damage | +50% |
| Offense | Fire rate | +10% basic shots/second | +50% |
| Offense | Targeting range | +10% targeting range and projectile lifetime | +50% |
| Resilience | Hull capacity | +20% base maximum hull | +100% |
| Resilience | Contact armor | 10% less enemy contact damage | 50% less |
| Resilience | Post-hit protection | +10% existing invulnerability duration | +50% |
| Energy efficiency | Battery capacity | +20% base capacity | +100% |
| Energy efficiency | Reserve efficiency | 5% less charger reserve per energy delivered | 25% less |
| Energy efficiency | Charging speed | +20% charger delivery rate | +100% |

Targeting reach extends with projectile lifetime, preserving speed. Contact armor applies only to enemy contact, rounding damage upward to a whole number; it does not reduce hazard damage. The shared existing post-hit protection window also follows hazard hits and shield blocks. Reserve efficiency applies to charger energy delivered both into the battery and to running modules. It does not alter charger reserve capacity, recovery speed/delay, battery drain, or module activation thresholds.

Provisional costs, identical for every node: next ranks 1–5 cost 10/20/30/40/50 salvage and 0/0/1/1/2 components. Purchases spend banked balances atomically, increment exactly one rank, and reject unmet prerequisites, maxed nodes, or either insufficient resource without changing ownership or balances. The purchase screen is accessible from the hub only; one fresh input buys one rank. Show current/next effect, rank, cost, and locked/affordable/unaffordable/maxed state. Keyboard 1–9 and mouse supported; Backspace returns to hub. Remain readable at 1120×720 and 640×480.

Owned ranks survive success, failure, hub returns, fresh launches, and R restarts within the app session. Disk persistence remains DRO-18. Start each attempt from pristine tuning, apply purchased permanent modifiers once, then layer temporary XP modifiers. Reset clears XP modifiers without removing permanent effects or capturing temporary tuning as a new base. Shopping does not heal/refill an active attempt; purchases become effective at next launch. Existing direct-combat validation without a campaign remains the base drone.

## Architecture

Pure `passives.rs` owns node IDs, rank validation, costs, transaction rules, and deterministic stat derivation. Campaign owns the passive tree next to the wallet. `upgrades/runtime.rs` derives effective permanent baseline from the captured pristine tuning on launch/restart and on temporary picks, then applies XP modifiers. `energy/flow.rs` accounts for the reserve-to-delivery factor at depletion boundaries. Existing mission transitions gain a paused passive screen, while `passives/scene.rs` renders the catalog and purchase feedback. Purchases use the same mission input arm/release protection as other menus.

## Validation

Test prerequisites, all five ranks, exact affordability, insufficient resources, maximum/duplicate input, atomicity, arithmetic limits; verify all nine modifiers, temporary stacking and clean reset across missions. Exercise energy conservation and long/split frame equivalence across reserve depletion. Compare a base and upgraded drone under the same deterministic encounter with no optional modules. Run cargo fmt --check, cargo test --locked, cargo clippy --all-targets, and cargo dev native checks. Record fixture limitations, native screen evidence, and measured comparison in docs/playtests.md. Commit logical increments, push branch codex/dro-15-passive-tree, and open PR against main.
