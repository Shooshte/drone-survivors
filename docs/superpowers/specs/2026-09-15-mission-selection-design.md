# DRO-17 mission selection and campaign unlocks

User-approved scope, September 15, 2026. Based on main `37cac67` after DRO-13, DRO-15 and DRO-16.

## Campaign and replay rules

Deliver 12 playable, individually identified missions arranged in three four-mission acts. In each act, the introduction unlocks two branch missions; completing both branches unlocks the finale. The finale unlocks the next introduction. Only mission 01 starts unlocked. A successful result marks that mission complete, once; failures and interrupted attempts do not unlock anything or erase success. All 12 distinct successes complete the campaign. Every unlocked mission remains freely replayable, including after campaign completion.

All 12 deliberately reuse the existing arena, 300-active-second survival objective, waves, chargers, hazards, enemies, XP and rewards without differentiation. Each successful attempt, including replays, retains collected loot plus 10 salvage and 1 component. Failure retains 25% of collected loot, rounded down, with no bonus. No first-clear bonus. Threat hints describe the same increasing chaser waves and finite chargers. The campaign placeholder therefore contains 60 minutes of successful combat before pauses/retries, not the eventual roughly 90-minute campaign target. Handcrafted maps and objective types are future work. The previous ticket requirement for different demands is superseded by the user's explicit instruction.

## Menus and state

Retain Hub → Briefing → Combat → Results → Hub. Add Choose mission from the hub (C or mouse), opening a paused selection screen. Show three acts with all four missions each, locked/available/completed state, prerequisites and the currently selected mission. Arrow keys cycle unlocked missions; clicking an unlocked card selects it. Enter or a briefing button opens its briefing; Backspace returns to the hub. Locked cards cannot select or launch and explain their prerequisites. Selection survives hub visits, failures, purchases and results. Every action requires released input before another action; ambiguous simultaneous inputs resolve to one action.

Briefing and results identify the specific mission. All mission metadata exposes the shared duration, threat hints and reward rules. Hub and selection display distinct completion progress and campaign complete at 12/12. Layout and controls work at 1120×720 and 640×480 using the established menu palette.

A typed MissionId and pure campaign progress object own prerequisites and completion. MissionSession holds selected mission and active mission snapshot; MissionResult records that snapshot. Launch rechecks unlock and loadout validity. Restart reuses active mission and loadout with a new attempt identity. Finalization records/rewards at most once, using the active mission rather than current menu selection. Permanent upgrades, wallet, inventory and mission progress remain session-only; save/resume stays DRO-18. Existing direct-combat validation fixtures retain their behavior.

## Verification and delivery

Test both branch orders in all acts, premature finale/next-act rejection, failure/restart/replay semantics, all-12 completion, selected/active identity isolation, stale input, pause boundaries and existing shop/lifecycle flows. Run cargo fmt --check, cargo test --locked and cargo clippy --all-targets --locked -- -D warnings. Use cargo dev for native menu/layout validation and ordinary navigation/launch/restart. Synthetic completion fixtures must be labeled clearly. Record evidence in docs/playtests.md, commit milestones, push and open a PR against main.
