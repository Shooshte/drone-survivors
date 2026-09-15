# DRO-19 — Reusable mission objectives

September 15, 2026. Implemented from approved scope on isolated branch
`codex/dro-19-objectives`, based on main `cdca2ed`.

## Delivered behavior

Mission 02 visits three unique scan sites then extracts. Mission 03 collects
three dedicated cargo then extracts. Cargo never enters the wallet. Both use
3D proximity and terrain line-of-sight, the current arena/combat, and the common
mission result/reward/campaign/save transaction. The other ten missions retain
survival. New objectives have elapsed time without a deadline victory; the
existing finite wave schedule still stops new bursts after five minutes.

Visible beacons, locked/ready extraction, compass/distance/height guidance and
objective-specific briefings make the placeholders playable. R resets progress
for the active mission; choices pause it. Death beats extraction; extraction
beats opening a new upgrade choice. Same-frame resource collection precedes
extraction, then settlement occurs once. Version-1 saves remain compatible.

## Automated checks

- `cargo fmt --check`: passed.
- `cargo test --locked`: **354 passed, 0 failed, 4 existing opt-in diagnostics ignored**.
- `cargo clippy --all-targets --locked -- -D warnings`: passed.
- `git diff --check`: passed.

[Full test output](dro-19-tests.txt) and [Clippy output](dro-19-clippy.txt).

Regressions cover repeated/partial visits, premature extraction, separate cargo
and resource accounting, fatal hull at a ready exit, success versus pending XP,
choice pause, active identity/restart, uncapped objective time, survival's unchanged
cap, 3D/line-of-sight checks, connected body-clear placeholder sites, target
guidance, briefing copy and cargo-result save/reload. Interrupted replay reloads
with zero cargo and never credits rewards again. All-twelve campaign tests now
complete the new types through actual objective proximity.

The deadline regression failed with Survived instead of Playing before the fix.
The same-frame pickup regression failed with 10 instead of 14 salvage before
explicit collection ordering. Both now pass.

## Native fixtures

Ran the documented `cargo dev -- --validate objectives` command twice, with
`DRONE_CAPTURE_DIR` and with/without `DRONE_CAPTURE_MINIMUM=1`:

- [640×480 output](dro-19-native-640x480.txt): passed.
- [1120×720 output](dro-19-native-1120x720.txt): passed.

Each exercises both mission types through selection/briefing/launch, premature
exit, repeated visit, elapsed time beyond 300, clean restart, death at a ready
exit, successful replay and results/hub. Each ends with four settled results,
20 salvage and two components. All visible UI text stays inside the viewport;
beacon/exit captures assert at least one rendered objective label.

These are **synthetic position and fatal-hull fixtures**, with mission 01
pre-unlocked and spawn bursts disabled. They exercise real objective and
settlement systems but do not measure difficulty, flying skill or pacing.

The updated [native campaign fixture](dro-19-campaign.txt) also passed at 640×480:
12 distinct victories, failure/success replays, 14 results and 130 salvage/13
components. Its success timers/objective progress and deaths are synthetic.

Screenshot inspection caught invisible Text2d labels in the Camera3d scene.
They were replaced with projected UI text; both fixtures were rerun and the
rendered labels inspected. Independent review confirmed the correction and the
resource collection order, with no remaining correctness findings.

## Ordinary keyboard/mouse check

Launched the same `cargo dev` build through a temporary app wrapper for computer
control, with `/tmp/dro19-manual-campaign.json`. Only mission 01 completion was
seeded, with zero funds and an empty loadout; the user's save was untouched.
Verified Continue → hub, mouse-opened mission selection, mouse-selected cargo
extraction, keyboard-opened briefing and launch, normal enemy spawning (five
hostiles at six elapsed seconds), cargo guidance and R restart (elapsed/hostiles
returned to zero). Closed the test app afterward. Full objective completions
were verified by the native fixture, not by a manual flight route.

## Captures

- [Compact scan briefing](../images/dro-19-640x480-scan-briefing.png)
- [Locked extraction](../images/dro-19-640x480-scan-locked-exit.png)
- [Cargo beacon](../images/dro-19-640x480-cargo-beacon.png)
- [Ready extraction](../images/dro-19-640x480-scan-ready-exit.png)
- [Cargo results](../images/dro-19-640x480-cargo-success.png)
- [Normal cargo briefing](../images/dro-19-1120x720-cargo-briefing.png)

## Remaining playtest work

No known blocking defect. Positions, radii and pacing are prototype values.
Guidance gives a direct compass bearing, not a route around walls; existing
passages/detours remain necessary. Natural completion time, difficulty and
loadout comparisons belong to the downstream vertical-slice playtests. Existing
Bevy shutdown logs emit a skipped-window-destroy event warning after successful
exit; no assertion, save or gameplay failure accompanied it.
