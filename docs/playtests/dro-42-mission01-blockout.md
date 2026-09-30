# DRO-42 — Mission 01 playable blockout

Date: 2026-09-26. [Linear issue](https://linear.app/drone-survivors/issue/DRO-42/mission-01-sketch-based-playable-blockout).
This is agent implementation and validation evidence. Human acceptance remains pending.

The initial sections below record the original September 26 implementation.
The September 27 playtest follow-up at the end supersedes its scale and pacing.

## Authored rules and tuning

- Seven full-height landforms follow the supplied square sketch, including the
  east-attached ridge, west passage, southern reward pocket and challenge area.
  Coordinates use 40 world units per sketch unit in a 40,000 × 40,000 arena.
  Forty-one AABB strips and a cached 36-anchor navigation graph support swept
  body collision and pursuit; altitude cannot bypass terrain.
- Mission 01 launches southeast, collects one payload by western proximity
  (600-unit radius), and delivers northeast (1,000-unit radius). There is no
  timer win. Actual movement segments order pickup, delivery and contact:
  delivery wins a simultaneous lethal hit; earlier lethal contact fails.
- Slot 4 is reserved before launch. Existing four-module saves retain equipment
  and ownership; briefing blocks launch with an instruction to clear that slot.
  Empty loadouts work. Missions 02–12 retain four available equipment slots.
- Groups 4a/4b each request ten chasers; group 6 requests twenty; guards 9/10
  each request five chasers and two slowers. Each activates once within 1,200
  units with clear sight. Fixed requests survive temporary cap/space saturation.
- Pickup enables permanent, indestructible sources 8 and 11. Each requests four
  enemies immediately and every five active seconds. Northwest uses chasers;
  northeast uses three chasers and one slower. Live enemies plus warnings cap
  at 96. Pending repeat demand coalesces to four per source; round-robin queue
  admission prevents source starvation without growing a backlog indefinitely.
- Hidden pocket: three components once. Challenge 7: five components after
  thirty active seconds inside; six chasers at 0/10/20 seconds. No kill quota.
  Early exit consumes the attempt, including a crossing that ends outside or
  an excursion returning inside in one frame. Reentry cannot restart it.
  Choices pause time; R starts fresh. Rewards remain subject to ordinary
  success/failure settlement. Overflow retries cannot duplicate collection.
- Four directional fields follow the sketch arrows. Horizontal travel is +40%
  with / −40% against the arrow, with unchanged perpendicular/vertical motion,
  no overlapping stack and no persistent velocity/stat modification.
- Four finite chargers reuse the Act 1 reserve of 200 each. Generic exploration
  caches and standalone XP pickup are absent from this authored map. Existing
  mission layouts, catalog scenarios, save format and other unlocks remain.

## Flight, clearance, pressure and fairness measurements

The main authored route is 68,832 world units. Actual rotor integration using
`FlightConfig::default()`, no equipment, altitude hold and an automated input
controller at 120 Hz completed start → pickup → delivery in **174.84 seconds**.
Every step checked rotated-body terrain clearance. Enabling the authored fields
also measured **174.84 seconds** because this route avoids them. This calibrates
scale; it is not a combat run or a human route-time promise.

A separate real-flight comparison crossed every field at an initial 200 units/s:
**280 with / 120 against / 200 perpendicular**, with unchanged vertical motion
and stored flight velocity. Passage checks cover eight headings and ±30-degree
bank/pitch envelopes, all landmark/charger approaches and alternative routes.
An actual enemy rotor pilot reached pickup and the hidden pocket from the
northwest source, rounding the ridge without terrain overlap.

The explicit full-ECS pressure probe warmed up for 70 simulated seconds to 96
live enemies, then measured 480 updates at a simulated 60 Hz. Actual pursuit and
control attacks ran; weapon targeting and contact damage were disabled to keep
population steady. CPU update durations: **min 0.974 ms, median 1.110 ms,
p95 1.304 ms, max 1.415 ms**. Repeat backlog stayed at or below eight. This is
CPU/ECS evidence, not a native rendering benchmark.

The starvation regression freed one slot at a time for 1,000 admissions over
5,000 active seconds: **500 enemies from each source**, with mixed batches
retaining their slower and bounded pending demand. Separate checks cover unsafe
spawn recovery, exact fixed group counts/types, kill XP, real R reset, holdout
pause/exit/reentry/completion and once-only optional collection.

The updated disk-backed campaign probe uses ordinary keyboard controls at 30 Hz,
normal contact damage and the real payload route. Empty-loadout Mission 01
completed in **182.40 seconds with 90 hull**, no kills and a 10-salvage/1-component
success payout. The empty, Overdrive and Shield branches then completed normal
Mission 02 after save/reload; Shield first earned one additional payload replay
to afford its 15-salvage price. No health, funds, wins, XP or objective progress
were granted. This scripted route supports basic playability and progression;
it does not establish satisfying encounter pressure or human acceptance.

## Native presentation checks

Both native fixtures completed at **1120×720 and 640×480 logical pixels** on the
local macOS/Metal host. Retina screenshots have twice the pixel dimensions.
They use the normal menu, rendering, enemies, weapons, upgrades, reward and
mission lifecycle. Disclosed overrides: scripted poses at landmarks, contact
damage disabled, earned choices skipped via Backspace, and no campaign save
plugin. They do not demonstrate a damage-enabled natural completion.

| Window | Active frame samples | Median interval | p95 interval | Maximum interval |
| --- | ---: | ---: | ---: | ---: |
| 1120×720 | 5,231 | 8.34 ms | 10.52 ms | 87.47 ms |
| 640×480 | 5,397 | 8.34 ms | 8.60 ms | 14.37 ms |

Intervals are `Time<Real>` during active gameplay after startup, include capture
stalls, and are not GPU timings. The maximum is retained rather than hidden.
Both runs reached payload delivery at about 46 active seconds and banked the
five-component holdout reward plus the ordinary success bonus. The harness
initially stalled at an earned upgrade; a failing regression led to normal
Pick/Skip input handling. An unsupported minimap arrow glyph was replaced.

Inspected captures show reservation/loading, terrain, field arrows, permanent
source markers, finite chargers, beam-warning HUD, holdout completion and results.

- [1120×720 ridge and map](../images/dro-42-1120x720-ridge.png)
- [1120×720 directional field](../images/dro-42-1120x720-directional-field.png)
- [640×480 launch and reserved slot](../images/dro-42-640x480-launch.png)
- [640×480 beam warning and completed holdout](../images/dro-42-640x480-holdout-complete.png)
- [640×480 settled delivery result](../images/dro-42-640x480-result.png)

Final automated verification: **531 tests passed, seven opt-in diagnostics
ignored** in the regular suite; the population-cap and disk-backed campaign
probes passed separately. Formatting and strict all-target Clippy passed.
Independent whole-branch review and follow-up found no remaining actionable
issue after the fixes described below.

Reproduce from the checkout:

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked mission01 -- --nocapture
cargo test --locked mission01_population_cap_full_schedule_performance_probe -- --ignored --nocapture
cargo test --locked vertical_slice_probe -- --ignored --nocapture
DRONE_CAPTURE_DIR=/tmp/mission01 cargo dev -- --mission01-check
DRONE_CAPTURE_MINIMUM=1 DRONE_CAPTURE_DIR=/tmp/mission01-small cargo dev -- --mission01-check
```

If launching a binary from a shared/external target directory, set
`BEVY_ASSET_ROOT` to this checkout so native assets resolve correctly.

## Review and compatibility

Review found and fixed a cached-navigation terminal approach that rejected an
exact-body-clear destination beside a ridge, a Bevy ordering dependency on a
multiply registered system, and old validation fixtures that still assumed
Mission 01 used survival timing, four equipment slots or shared caches.
Regression fixtures now select an explicit survival placeholder where needed;
objective/exploration wave suppression runs after mission baseline configuration.
Native mission fixtures install the authored map renderer. Native `missions`,
`passives`, `shop`, `objectives`, `campaign` and `exploration` checks all passed
at 1120×720. The exploration rerun also checked retained battery state in the
hub after results copy moved to next-action guidance. These existing fixtures
retain their disclosed synthetic setup/outcomes and never access the user save.
Save tests cover
payload settlement and preservation of legacy equipment without a schema change.

## Remaining human acceptance

DRO-42 authorizes these Mission 01 additions; it does not establish the older
DRO-21/DRO-22 human acceptance gates. Numeric tuning remains provisional.
A human should still assess:

- Whether the map communicates pickup/delivery and full-height ridge routes,
  including charger access and the optional southern pockets.
- Whether an ordinary empty/three-module run has satisfying combat pressure,
  travel time, reward value and source pacing after pickup.
- Whether Slowing Beam warnings and cover/range counterplay are readable while
  steering, especially near the delivery guards.
- Whether all four arrow directions and with/against field feel are intuitive.
- Whether the one-attempt holdout boundary and early-exit cost are clear.

These are acceptance questions, not claims resolved by synthetic screenshots or
headless performance measurements.

## PR review follow-up: briefing and live charger feedback

Both reported issues were valid. Mission 01's briefing note now states the
actual hidden-reward proximity and component amount from the gameplay constants
(350 units, three components, no XP). Other missions keep their shared-cache
instructions. Launch rejection feedback and settlement/restart guidance remain.

The six existing charger assemblies now follow their live charger entities.
Their local field, core, rings and reserve gauge move together on map changes;
two disabled Mission 01 nodes hide their complete assemblies. The authored map
no longer hides these visuals or draws duplicate static charger rings. Charging
still selects the original active material, and depletion empties the gauge
inside its persistent outline. Reset and returning to a shared mission restore
normal reserve displays without allocating new scene assets.

Both regressions failed before the fixes. The new scene checks cover map-specific
briefing notes/feedback, Mission 01 → Mission 04 → Mission 01 relocation and
visibility, real recharge/highlight, depletion, reset and stable gauge/mesh counts.
Final verification: **533 passed, seven opt-in diagnostics ignored**; formatting,
strict all-target Clippy and independent follow-up review passed.

The native fixture now captures briefing and occupied charging/depleted states
before delivery and exits at about 58 seconds. Its additional overrides are an
empty player battery and an explicitly emptied charger reserve, solely to show
both visual states; the existing scripted poses, zero contact damage and absent
save plugin remain. These captures are presentation evidence, not balance data.

Native follow-up passed at 1120×720 and 640×480, with the charging highlight and
partially filled gauge visibly distinct from the idle depleted field and empty
outline. These two presentation runs were concurrent, so their frame intervals
are not compared with the earlier performance measurements.

- [Corrected compact briefing](../images/dro-42-640x480-briefing.png)
- [Compact charger supplying power](../images/dro-42-640x480-charger-charging.png)
- [Compact depleted charger and empty reserve gauge](../images/dro-42-640x480-charger-empty.png)


## September 27: compact map and playtest tuning

User feedback identified unclear holdout instructions, safe camping before
pickup, weak pursuit/beam pressure, difficult scrap collection and empty travel.
The approved iteration retains scale **12** (12,000 × 12,000 world units), down
from 40. A terrain-clear rotor route measures **53.37 seconds** without combat,
with or without the authored fields, which this route avoids.

- Four-chaser patrols request ordinary 0.75-second spawn warnings after three
  active seconds and every eight seconds thereafter. Safe candidate positions
  lie 550–850 horizontal units from the player, with enemy hull altitude clamped
  inside the arena and safety checked again when the warning activates.
- Mission 01 Chasers use a **340u/s cap**. The shared thrust taper produces about
  **320.28u/s** in straight pursuit, compared with 244.92 previously. Scout cap
  remains 420, beam multiplier 0.6, and other enemy kinds/maps retain their tuning.
- Each permanent source requests four enemies immediately after pickup, then
  every **5 seconds** initially, **4 seconds at 20s**, **3 seconds at 40s**, and
  **2 seconds from 60s** carrying. Pause freezes clocks. Restart clears them.
  All live enemies and warnings share cap 96. Fixed groups persist; repeat
  demand coalesces to at most twelve pending enemies across sources and patrols.
  A one-slot fairness test admitted 333 enemies from each repeat lane over 999
  admissions while preserving exact authored group counts and mixed batches.
- Ordinary Mission 01 salvage attracts within **300 horizontal units** at any
  legal flight height. Swept flybys count; visibility uses the actual 3D flight
  path. Walls still prevent homing/collection. Existing 900u/s homing, once-only
  credit, settlement and other missions' 100u 3D/cache behavior are unchanged.
- Holdout radius is **840**. The map and briefing state optional, stay 30s and
  +5 components. Nearby guidance adds the early-exit cost; an active countdown
  and progress bar show remaining time. Completed/forfeited states are explicit.

### Damage-enabled pressure measurements

The 60 Hz full-plugin stationary/moving probe uses ordinary firing, enemy
attacks, energy, terrain, encounters and objective logic with no tuning overrides.
Upgrade offers are skipped through normal controls. The moving pilot uses legal
keyboard steering, without pose teleports or invulnerability.

| Pilot | First warning | Enemy within 400u | First damage | Outcome |
| --- | ---: | ---: | ---: | --- |
| Stationary Scout | 3.00s | 5.47s | 7.20s | Died at 31.20s; 14 kills |
| Moving Scout | 3.00s | 6.00s | 12.22s | Delivered at 56.80s; 60 hull; 8 kills |

A controlled beam probe disables player shooting and places one Slower 180u ahead
and one Chaser 650u behind the Scout. Natural movement and beam windup remain.
After holding still 1.33s, forward movement produced 1.67s of slowed flight;
the Chaser gap closed **65.22u** (557.06→491.84). This demonstrates meaningful
closing pressure in that setup, not a universal difficulty or human acceptance claim.

The capped 96-enemy CPU/ECS probe, with contact damage and weapon targeting
disabled, measured 480 updates: median **0.650ms**, p95 **0.781ms**, max **0.897ms**.
These are schedule timings, not rendering FPS.


Final verification: **546 passed, nine opt-in diagnostics ignored** in the locked
suite. The disk-backed campaign probe passed separately for empty, Overdrive and
Shield loadouts; formatting, strict all-target Clippy and diff checks passed.
Independent review found a shared warning-activation limit of 30 that could
cancel patrols below Mission 01's cap. Both admission and activation now use the
same 96-enemy constant; a full-plugin 30→34-live regression also confirms that switching
to another mission restores its default cap. Follow-up review has no remaining
findings.


Both final native fixtures completed successfully at 1120×720 and 640×480. The
briefing and holdout approach/countdown/completion text fit the tested layouts.
They retain the disclosed scripted positions, zero contact damage, synthetic
charger/battery states and absent campaign save. Concurrent runs are presentation
checks; their timing is not used for performance comparison.

- [Compact briefing](../images/dro-42-tuned-640x480-briefing.png)
- [Holdout instructions before entry](../images/dro-42-tuned-640x480-holdout-approach.png)
- [Compact countdown and progress](../images/dro-42-tuned-640x480-holdout.png)
- [Completed reward](../images/dro-42-tuned-640x480-holdout-complete.png)
- [Full-size countdown and map](../images/dro-42-tuned-1120x720-holdout.png)

These checks support another human playtest; balance acceptance remains pending.
