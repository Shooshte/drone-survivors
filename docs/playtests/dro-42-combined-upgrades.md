# Mission 01 + twelve upgrades: combined playtest

September 29, 2026. The user authorized enabling all twelve temporary upgrades and
Repair/Repulsor alongside DRO-42 for human testing. This extends the earlier
catalog-only restriction for those features; human acceptance remains pending.
The existing permanent passive tree remains accessible from the hub with U.

## What changed

Mission 01 earns offers from the twelve-card pool. Other missions retain six.
Eligibility still follows equipped modules; there are four opportunities per run,
up to three options per offer, and no preview XP in normal campaign play.
Choice effects use the existing pristine baseline plus permanent progression.
Restart removes temporary effects; the payload still reserves slot 4.

Repair and Repulsor each cost 15 salvage in the campaign shop. Both retain their
existing power costs, healing/pulse mechanics and upgrade tradeoffs. Ownership
and equipped slots save through the normal campaign flow. Old version 1 saves
remain valid and do not receive free modules. New saves can contain either
support module; older builds that do not recognize them cannot read those saves.

## Human playtest

1. Run the DRO-42 workspace normally with `cargo dev`.
2. Earn and bank salvage by completing Mission 01. The optional holdout pays
   five components and provides extra combat time for earning upgrade choices.
3. Open the module shop with M. Buy Repair and Repulsor for 15 salvage each;
   optionally buy Overdrive for 10. Equip slots 1–3, leaving 4 for the payload.
4. Replay Mission 01. Earn XP, read the benefit/drawback on each offered card and
   try the new options. Support-specific cards require the corresponding module.
5. Compare a second loadout. Check energy pressure, healing/push usefulness,
   slowing-beam pursuit, holdout readability and whether the upgrade choices
   arrive frequently enough on the shorter map.
6. Quit/reopen and Continue to check that bought modules and equipment persist.

The direct route can finish before many upgrade opportunities are earned. XP
costs remain 50/150/300/500 (cumulative 50/200/500/1000); frequency is a playtest
question, not a claim resolved by injecting XP in automated fixtures.

## Agent verification

The focused Mission 01 fixture covers earned XP opening an expanded offer with
no catalog preview, all twelve definitions reachable across legal zero-to-three
module loadouts, prerequisite filtering, all four choices, reset, and returning
to an ordinary six-card mission. Temporary grants in these tests are explicit.
Shop/save coverage checks exact spending, duplicate purchase rejection, six-way
keyboard navigation, menu purchase/equip/save/Continue and old-save loading.

The combined native fixture runs with:

```sh
DRONE_CAPTURE_DIR=/tmp/combined cargo dev -- --mission01-upgrades-check
DRONE_CAPTURE_MINIMUM=1 DRONE_CAPTURE_DIR=/tmp/combined-small cargo dev -- --mission01-upgrades-check
```

It grants 40 salvage and 1,000 XP, drives ordinary shop/choice input, disables contact
damage and scripts pickup/delivery positions. It never installs SavePlugin.
Screenshots and bounds checks run after UI layout; incomplete steps time out.
It checks live changed stats and complete reset, but does not measure natural
progression, player preference or difficulty. The separate shop fixture also
uses disclosed synthetic funds and a synthetic success timer.


## Results

- `cargo test --locked`: **554 passed, nine opt-in diagnostics ignored**.
- Formatting, strict all-target Clippy and diff checks passed.
- Independent review found no actionable defects in the combined integration.
- Native combined fixture passed at **640×480 and 1120×720**. Both runs selected
  Wide repulsor, Efficient coils, Reserve battery and Rapid repair through normal
  offered cards; live modifiers changed, restart restored the baseline, support
  equipment remained, and payload delivery succeeded.
- Shop and choice text stayed inside the tested viewports. Visual inspection
  confirmed readable six-module shopping, payload slot reservation, benefit and
  drawback copy, and the paused mission behind the choice screen.
- A Bevy window-cleanup warning appeared after the successful native exits.

The small shop initially overflowed after adding two rows. Tighter panel spacing
fixed it. The combined fixture verifies both support purchases through ordinary
shop input before entering Mission 01.

[Support loadout in the compact shop](../images/mission01-upgrades-640x480-shop-equipped.png)

[Mission 01 offer with Wide repulsor](../images/mission01-upgrades-640x480-choice-1.png)

[Efficient coils, Hot overdrive and Rapid repair](../images/mission01-upgrades-640x480-choice-2.png)

[Large-window choice over Mission 01](../images/mission01-upgrades-1120x720-choice-3.png)


The rebuilt compact shop fixture also passed all six purchases and lifecycle
steps after a deliberate six-second renderer/process stall. That reproduction
caught absolute test deadlines sending adjacent inputs without release frames;
relative scheduling plus an explicit release frame fixed the test driver.
Production input and purchase rules were unchanged. Strict Clippy remained clean.
