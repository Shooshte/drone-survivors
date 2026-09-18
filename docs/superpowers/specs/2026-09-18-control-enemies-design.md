# DRO-36: Slowing beams and module jammers

Scope: catalog-only enemy variants, standalone and mixed scenarios. Campaign
waves, shops, offers and saves remain unchanged. User authorized the next queued
issue through PR delivery; these unspecified numeric mechanics are provisional.

Use one component state machine (approach, windup, active, recovery) for both
ranged enemies. Compared with instantaneous proximity effects, windup offers
counterplay; compared with projectiles, a sight-checked beam needs no additional
collision system. Both use existing rotor flight, navigation, health (40),
weapons, contact damage and kill accounting. They approach to 240 units then
hold a position while attacking; attack range is 320 units in 3D. Windup takes
1.2 active seconds, requires continuous clear sight and range, and cannot be
skipped by a long frame. Losing sight/range cancels into 3 seconds of recovery.
Destroying the source cancels a pending attack or active slowing beam.

Slower: 2 seconds active, then 3 seconds recovery. The beam reduces horizontal
speed cap and acceleration to 60%; vertical thrust assistance and steering stay
available to escape. Multiple beams still give 60%, never multiplication.
Movement config remains derived from current upgrades and Mobility. Clamp
existing horizontal velocity to the reduced cap before integrating motion.
There is no lingering slow once sight/range breaks or the beam ends.

Jammer: announce one equipped slot during windup, preferring enabled modules,
then lowest slot. Target never changes mid-warning. At delivery, force that
slot OFF and lock it for 3 active seconds. Reject toggles while locked. The
pulse has a 0.4-second active visual, then recovery. Locks do not extend/stack:
only one slot can be locked at once, followed by 3 seconds global jam immunity.
A delivered lock expires even if its source dies. Expiry leaves the slot OFF;
player must press its key again with the ordinary battery threshold. Shield
blocks damage, not control attacks. Basic fire and steering always remain.

Compute attack/status state before player movement and power preparation.
Timers freeze outside gameplay; reset/arena return clears all effects. Existing
energy staging must preserve the lock state and cannot charge disabled modules,
recharge their shields, launch rockets or confer Mobility/Overdrive benefits.

Visuals: turquoise crossed emitter for slower, purple antenna for jammer.
Yellow expanding warning ring and thin sight line; solid active beam/pulse;
blue recovery ring. Compact catalog HUD names windup/active/recovery and targeted
slot; module row shows LOCK and remaining seconds, then OFF. Text and geometry
provide cues beyond color alone. Three additional scenarios: Slowing beam,
Module jammer, Mixed control (both plus a fast pursuer).

Validation: production-system tests across frame rates for state transitions,
range/cover/death, bounded overlaps, module power/effects, pause/reset and arena
selection. Native fixture uses synthetic positioning and suppressed auto-fire
only to expose cues at 1120x720 and 640x480. Record its limits separately from
human playtesting. Run full tests, formatting and Clippy before PR.
