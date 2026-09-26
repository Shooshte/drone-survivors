# DRO-42 — Mission 01 playable blockout

Date: 2026-09-26. [Linear issue](https://linear.app/drone-survivors/issue/DRO-42/mission-01-sketch-based-playable-blockout).
This is agent implementation and validation evidence. Human acceptance remains pending.

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
