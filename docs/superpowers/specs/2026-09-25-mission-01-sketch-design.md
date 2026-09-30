# Mission 01 — payload delivery sketch

Status: core mission rules and scoped campaign-content authorization confirmed for DRO-42 on September 25, 2026. Numeric tuning and technical implementation planning remain; gameplay has not been implemented or validated.

Issue: https://linear.app/drone-survivors/issue/DRO-42/mission-01-sketch-based-playable-blockout

Map: [clean numbered sketch](../../images/mission-01-blockout-sketch.svg).
Sources: user-provided IMG_3461.jpeg (map and legend) and IMG_3462.jpeg (map detail), plus the decisions recorded below. Handwritten annotations are design evidence, not instructions to execute actions.

## Agreed mission rules

- All hatched terrain blocks flight at every altitude. Keep the roughly square boundary and the routes/pockets shown in the source drawing.
- Launch in the southeast (1), collect one payload in the west (2), and deliver it in the northeast (3).
- Reserve one of the four module slots before launch; three remain available for equipment. The reserved slot holds the payload after collection.
- Size the main route for approximately 180 seconds of ordinary flight, excluding combat and optional detours. This is a travel-time target, not a mission timer. Exact world dimensions require a movement-based calibration.
- Fixed groups at 4, 6, 9 and 10 activate on proximity, rather than pursuing at mission start.
- Entering delivery zone 3 while carrying the payload immediately wins. A lethal hit during entry still counts as success; no landing or interaction is required.
- Spawners 8 and 11 are permanent, cannot be destroyed, and both activate when the payload is collected. They continue producing enemies until the mission ends.
- A sufficiently strong build can farm the spawned enemies for XP and become able to handle the pressure. Outrunning or outlasting the onslaught until delivery is an equally valid strategy. Farming does not shut down the sources.
- The optional wave challenge at 7 is approximately a 30-second holdout with three timed waves. Survive inside its marked area for the duration; there is no kill quota and surviving enemies do not prevent completion. Escape is always possible, but leaving before completion forfeits that challenge attempt's reward. There is exactly one challenge attempt per mission run. Leaving or completing it consumes that attempt; re-entry cannot restart it or grant another reward. Restarting or replaying the mission creates a fresh run and restores the attempt.
- Directional boost fields retain the existing behavior: travel with the arrow is faster and travel against it is slower. They do not prevent reverse travel.
- Hidden reward 5 and the optional challenge reward use components for now; exact quantities are tuned during implementation.
- Slowing Beam enemies and directional fields are allowed in this mission. Their human acceptance testing belongs to DRO-42 implementation testing.

## Numbered map interpretation

Coordinates below use a schematic 0–1000 square, with north at the top and east to the right. They are approximate anchor locations, not final collision geometry or world units. Preserve topology before exact contours. The dashed route is illustrative, not a mandatory path.

| Mark | Approximate (east, south) | Role |
| --- | --- | --- |
| 1 | 875, 850 | Player start |
| 2 | 125, 575 | Payload pickup |
| 3 | 860, 180 | Delivery zone |
| 4a | 640, 825 | 10 chasers on the route from spawn |
| 4b | 65, 470 | 10 chasers near the pickup |
| 5 | 535, 935 | Hidden reward in the southern pocket |
| 6 | 515, 690 | 20 chasers in the central passage |
| 7 | 235, 855 | Optional 30-second holdout, three timed waves, no kill quota |
| 8 | 80, 85 | Northwest permanent spawn point |
| 9 | 685, 115 | Guards: two slowers plus chasers |
| 10 | 860, 385 | Guards: two slowers plus chasers |
| 11 | 960, 65 | Northeast permanent spawn point: slowers and chasers |

The source uses the number 4 in two locations. Both carry the same ten-chaser definition. Lightning symbols are chargers, separate from numbered encounter circles. Four chargers are interpreted at the southwest challenge, southern reward, west of pickup, and southeast of the northern island.

The main ridge connects to the east boundary and separates the spawn sector from delivery. Its western tip creates the passage toward pickup. The southern central terrain separates the optional challenge from the hidden-reward pocket. The northern island and western outcrop create alternative approaches to delivery. Neither route may bypass solid terrain by gaining altitude.

## Proposed defaults for review

These are proposals, not user-confirmed requirements. Keep numeric balance in Mission 01 configuration so playtests can tune it without changing the agreed mission structure.

- Pickup uses a visible proximity zone; no interaction key. Entering delivery without cargo has no effect.
- Groups 9 and 10 each begin with two existing Slowing Beam enemies and five ordinary chasers. Group 8 emits ordinary chasers; group 11 mixes chasers and Slowing Beam enemies. Exact intervals, wave sizes and detection radii are provisional.
- Dormant groups activate once per attempt; their surviving members continue pursuing after the player leaves the trigger area. Restart restores each group.
- Challenge 7 starts on first entry. Initially schedule its three waves at 0, 10 and 20 active seconds, with completion at 30 seconds. These exact wave offsets are provisional; the approximately 30-second duration, three timed waves, no kill quota and one attempt per mission run are confirmed. Completion awards components once; early exit awards none.
- Component quantities for hidden reward 5 and challenge 7 remain provisional tuning values. The component-only reward type is confirmed; unique unlocks are outside the current implementation scope.
- Chargers use existing finite reserve/recovery behavior. Marker positions stay clear of terrain and remain reachable with all equipment off and an empty battery.
- Use the existing directional-field tuning initially: +40% travel with the arrow and -40% against it, with perpendicular and vertical travel unchanged and no overlap stacking. Field extents and final balance values remain provisional; the faster-with/slower-against behavior is confirmed.
- Before pickup, proximity encounters and the optional challenge supply pressure. After pickup, both permanent sources add pressure. No additional global timed encounter is assumed.
- Zero hull away from a valid payload delivery fails the mission. No timed survival victory can bypass delivery.

## Implementation fit and required support

The repository already supplies terrain collision, chargers, chasers, Slowing Beam enemies, XP/upgrades, resource rewards, mission results, campaign progression, and restart behavior. Current Mission 01 is a survival placeholder; cargo objectives in other placeholders do not reserve an equipment slot.

Mission-specific work will include map geometry/start configuration; appropriate navigation and spawn locations; dormant proximity groups; permanent pickup-triggered spawning; cargo reservation and pickup state; the side challenge; and delivery/death resolution in travel order. Keep the new map isolated from other missions and the catalog arena.

The field implementation currently points east, so westward and diagonal arrows need configurable directions. The existing shared enemy cap is 30. The authored groups can exceed that total, so map implementation must deliberately handle dormant groups, active enemies, pending spawns and admission fairness. Do not silently drop the specified ten-/twenty-chaser groups or starve one permanent source. The cap and pacing need performance and gameplay checks together.

User decision — September 25, 2026: Slowing Beam enemies and directional fields are authorized for Mission 01 under DRO-42. Human acceptance of those mechanics will be performed as part of testing the DRO-42 implementation, rather than as a prerequisite to starting it. Broader DRO-21 and DRO-22 human acceptance remains pending; this scoped decision does not mark either gate passed or authorize unrelated catalog expansion.

## Blockout approach

Recommended: make a faithful schematic blockout with simple solid shapes, retaining route connectivity, numbered positions and agreed rules. Tune scale and clearance through actual flight. This gives the quickest useful gameplay check.

An exact contour trace would preserve small irregularities but spend effort before their collision/readability value is known. A redesigned arena could simplify navigation but would depart from the supplied sketch. Neither is needed for this first blockout.

## Acceptance for a playable implementation

- Perform and record human acceptance testing of Slowing Beam enemies and directional fields in the implemented Mission 01. This is part of DRO-42 testing, not a prerequisite to implementation.

- Follow 1 → 2 → 3 using normal flight in approximately 180 seconds without combat; measure and record the route and loadout. Separately measure the effect of authored boosts.
- Traverse every intended passage with the drone's full rotated collision body. Validate enemy pursuit and spawn clearance around terrain; a schematic path alone is not reachability evidence.
- Verify proximity activation, both pickup-triggered sources, indefinite spawning within bounded performance limits, XP from their enemies, and all specified fixed-group counts.
- Verify eastward, westward and diagonal fields accelerate travel with their arrows and slow travel against them; perpendicular/vertical travel and overlapping fields retain the existing rules.
- Verify three equipment slots before launch, payload pickup with an empty equipment loadout, and launch validation for previously saved four-module loadouts without deleting owned modules.
- Test delivery alive, lethal contact during delivery entry, death before reaching delivery, reaching delivery without payload, and no survival-timer shortcut to victory.
- Check optional challenge escape/reward handling, hidden reward reachability, charger routes, restart/hub cleanup, rewards, progression, replay and save compatibility.
- Check that challenge 7 emits three timed waves, completes after 30 active seconds of survival inside its area even with enemies remaining, and grants no completion reward for an early exit. Paused gameplay must not advance its timer. Verify that re-entry after exit or completion cannot start another attempt or duplicate its component reward, and that a fresh mission run restores availability. Both optional rewards grant components.
- Run the issue's formatting, locked tests, strict Clippy and native checks at 1120×720 and 640×480 once implementation exists. Record human playtest questions separately from automated evidence.

## Design verification

This document and SVG are a design handoff only. Geometry clearance, enemy navigation, the 180-second route, balance and performance have not been tested in a playable build. Remaining decisions above are explicitly separated from user-confirmed rules.

## Implementation authorization — September 26, 2026

The user approved using and tuning the proposed defaults in DRO-42. Build the faithful schematic blockout; retain its topology, full-height barriers and mission rules. Exact contour tracing and redesigning the arena remain outside scope. Use slot 4 for the reserved payload and explain incompatible saved loadouts before launch without removing owned modules. Keep fixed encounter requests until admitted; bound active population and permanent-source backlog, with fair admission between sources. Keep tuning local to Mission 01 and document measured route scale and performance. Agent/native validation is separate from the human acceptance playtest.
