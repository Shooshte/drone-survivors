# DRO-36 — Slowing beams and module-disabling enemies

September 18, 2026. Agent verification on native macOS. Human counterplay and
balance acceptance remain pending; catalog content remains excluded from campaign.

## Rules and counterplay

Three added scenarios: Slowing beam, Module jammer, Mixed control. The mixed
scenario has two bursts of beam/jammer/fast pursuer at 1 and 12 active seconds.
Production spawn warnings, clearance and population caps remain in use. No
campaign waves, save schemas, module shops or upgrade offers changed.

Both new enemies have 40 hull, use the existing flight/navigation integrator,
approach to 240 units and hold an anchor while attacking. The warning lasts at
least 1.2 active seconds and the attack cancels beyond 320 units in 3D or when
terrain blocks sight. Cancelling, completing a beam, or completing a jam pulse
starts 3 seconds of recovery. Killing a source prevents further attacks. Contact
uses ordinary damage/shield rules; control attacks bypass damage shields.

The cross emitter's active beam lasts 2 seconds and scales horizontal speed cap
and acceleration to 60%, with no stacking. Existing horizontal velocity is capped
before integration. Steering and vertical flight remain available. Effects are
derived from the current upgraded/Mobility configuration rather than permanently
changing it. Sight/range loss removes the slow on the next gameplay update.

The antenna's windup names the selected slot: first enabled, then first equipped.
It never retargets that warning. Delivery forces the slot OFF for 3 active seconds;
its effects, drain and shield recharge stop. Toggles are ignored during the lock.
There can be only one lock, it cannot refresh, and 3 seconds of global immunity
follow expiry. The slot stays OFF until a new key press with sufficient battery.
A delivered lock survives source death only for its remaining duration. Empty
loadouts cannot be jammed. Basic fire remains available without energy.

## Verification approach

Production-system tests cover timing at 30/60/144 Hz, a warning on a large frame,
range/sight cancellation before and during beams, source death, fixed target slot,
empty loadouts, overlap bounds, power and shield rejection, Mobility interaction,
baseline preservation, pause and restart, typed kills, and scenario selection.
The full existing campaign/save suite is included in regression checks.

An independent reviewer found that the initial attack acquisition threshold
anchored enemies at 320 units instead of approaching to 240. A regression test
starting both types at 280 units reproduced it; acquisition now uses 240 while
320 remains the sustain/cancellation range. Re-review found no remaining
actionable findings.

- `cargo fmt --check` and `git diff --check`: pass.
- `cargo test --locked`: **417 passed, 5 existing probes ignored, 0 failed**.
  [Full output](dro-36-tests.txt).
- `cargo clippy --locked --all-targets -- -D warnings`: pass.
  [Output](dro-36-clippy.txt).

Builds reused dependencies with
`CARGO_TARGET_DIR=/Users/shooshte/projects/drone-survivors/target`; source edits
and commits stayed in the separate DRO-36 worktree.

## Scripted flight comparison

`cargo test --locked control_keyboard_escape -- --nocapture` compares two
three-second runs per type at each frame rate. The source begins at (-240,90,0),
the drone at (0,90,0). Basic auto-fire is suppressed to isolate evasion. No
positions change after setup, no extra energy/health is granted, and production
flight runs throughout. Held E banks/turns the drone away from the source.

| Rate | Idle: time slowed in 3s | Banking: time slowed | Idle jam | Banking jam |
| --- | --- | --- | --- | --- |
| 30 Hz | 1.767s | 0s | Yes | No |
| 60 Hz | 1.783s | 0s | Yes | No |
| 144 Hz | 1.792s | 0s | Yes | No |

The banking route ends near (30,90,699); the source's warning cancels because
range breaks. Separate wall tests verify cover cancellation during warning and
active beams. These measurements establish one repeatable response; they do not
establish fair difficulty, every route's viability, or human cue readability.
[Probe output](dro-36-control-tests.txt).

## Native fixture

```sh
DRONE_CONTROL_SMOKE=1 DRONE_CAPTURE_DIR=/tmp/control cargo dev --locked -- --validate catalog
DRONE_CONTROL_SMOKE=1 DRONE_CAPTURE_MINIMUM=1 DRONE_CAPTURE_DIR=/tmp/control-small cargo dev --locked -- --validate catalog
```

This opt-in catalog fixture uses synthetic keys to select/launch all three new
scenarios. It suppresses basic auto-fire and repositions each single source once
after production spawning, clearing that source's attack phase. It grants no
health, energy, damage, XP or results. It checks beam warning/active/recovery,
jam warning/target/lock, zero active drain during lock, expiry to OFF, manual
reactivation, restart and the mixed roster. Visible text bounds are asserted at
every capture. Compact visual inspection caught a navigation marker overlapping
charger text after the extra status rows. Marker placement now reserves the
measured HUD height plus a line of slack, and hides markers when no safe area
remains. A unit regression and an overlap assertion on every native gameplay frame cover
this layout. Newly visible HUD rows defer marker placement until their first
layout completes; subsequent row changes retain one line of slack.
This is presentation/behavior evidence, not a fair-play encounter.

Both final runs passed at **1120×720** and **640×480**, including every-frame
navigation/HUD clearance and capture-time viewport checks. Selected Retina
screenshots were visually inspected for wrapping, silhouette, beam, lock and
recovery cues. Both exits emitted the existing winit “Skipped event Destroyed”
warning after the fixture PASS; exit status was zero.

- [Normal native log](dro-36-native-1120x720.txt)
- [Compact native log](dro-36-native-640x480.txt)
- [Compact selector](../images/dro-36-640x480-beam-selector.png)
- [Compact active beam](../images/dro-36-640x480-beam-active.png)
- [Compact locked module](../images/dro-36-640x480-jam-lock.png)
- [Compact expiry to OFF](../images/dro-36-640x480-jam-unlocked-off.png)
- [Normal active beam](../images/dro-36-1120x720-beam-active.png)

## Human follow-up

- Fly each standalone scenario with empty and equipped slots. Confirm the
  silhouettes and phase/slot cues give enough information before delivery.
- Evade warnings using banking, altitude and cover; try destroying the source.
- Test a shield-heavy and Mobility-heavy build in Mixed control, including a
  jammed module during a naturally earned upgrade choice and low battery.
- Compare whether 1.2-second warnings, 2-second beams and 3-second lock/immunity
  windows leave enough choice. Adjust numeric tuning from those observations.
- Keep campaign introduction behind the separate human gate.
