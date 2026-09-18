# DRO-38 — Directional fields and hull-repair sites

September 18, 2026. Agent verification on native macOS. Human handling/balance
acceptance remains pending; this content is isolated to the catalog arena.

## Implemented rules

Environment practice is empty; Environment pressure spawns Fast + Slower at
1 and 15 seconds with the ordinary warning/cap rules. Both add two eastward
fields centered at (-420, 150, -190) and (-420, 150, 190), each with half extents
(220, 150, 125). North stays active, south cycles six active seconds on/four off.
Blue/purple floor outlines, full-height corner posts and east chevrons mark them;
off chevrons disappear and the border turns gray. Text states lifetime, direction,
strength, countdown and player membership. Enemies are unaffected.

Within a field, east displacement is 1.4x and west 0.6x; perpendicular and vertical
travel remain 1x. Membership uses the drone center and is sampled per bounded
physics substep (at most 1/120 second). The boost feeds into swept terrain
resolution and the actual player path; stored velocity and flight tuning remain
unchanged. Exit/cycle-off response takes at most one substep, with no lingering
player modifier. Union membership prevents overlapping fields multiplying effects.
Mobility and active beam slow compose through ordinary flight before this modifier.

The repair site at (-420, 90, 0) has a 70-unit 3D radius and requires line of sight.
It supplies one charge restoring min(35, missing hull). Full hull preserves the
charge; subsequent damage while still inside can consume it. Swept path checks
catch fast crossings. Repair runs after combat, never revives a dead player and
clamps to the current upgraded maximum. A green cross/ring becomes a gray X/ring
when used, with repaired amount in the HUD. This is a map site, not the future
Repair module. Upgrade choices pause repair and the field cycle. Launch/restart
restores charge/cycle; return and ordinary scenarios remove fixtures.

## Automated evidence

Twelve new behavioral tests cover:

- East/west strength, neutral axes, finite height and overlap bounds.
- Field exit with unchanged velocity at 30/60/120 Hz and terrain collision/path.
- Six-on/four-off boundaries, full-hull preservation, one charge, restart and cap.
- Real app movement with powered Mobility, active beam and both together.
- Pause during upgrade, return/launch/reset, scenario selection/isolation.
- Swept repair crossing, height/LOS rejection, upgraded maximum and lethal bomb
  precedence; ordinary combat does not install the environment resource.

The initial selector/repair tests failed before implementation; the production
movement test failed before field integration. All pass with the feature.
A read-only reviewer found no actionable important correctness/regression issue
in gameplay and cues. No campaign/save schema or wave/shop/upgrade content changed.

Final verification: `cargo fmt --check` and `git diff --check` passed;
`cargo test --locked` passed **459 tests, 0 failed, 5 existing probes ignored**;
`cargo clippy --locked --all-targets -- -D warnings` passed.
See [test output](dro-38-tests.txt) and [Clippy output](dro-38-clippy.txt).
Builds reuse the repository target directory;
source edits and commits stay in the separate DRO-38 worktree.

## Native evidence

```sh
DRONE_ENVIRONMENT_SMOKE=1 DRONE_CAPTURE_DIR=/tmp/environment cargo dev -- --validate catalog
DRONE_ENVIRONMENT_SMOKE=1 DRONE_CAPTURE_MINIMUM=1 DRONE_CAPTURE_DIR=/tmp/environment-small cargo dev -- --validate catalog
```

Both **640×480** and **1120×720** fixtures passed. The fixture drives synthetic
keys, teleports the drone, sets hull to 40 and grants 50 XP to open a choice.
Auto-fire is suppressed. It observes real 40→75 repair, rejects repeat healing,
waits for the real off cycle, checks pause and restart, observes pressure spawns,
and returns to an ordinary scenario with no environment fixtures. Text bounds
are checked at every capture. This is rule/presentation evidence, not a human
playtest or difficulty comparison. Existing winit shutdown warnings occur after
PASS and do not change the exit status.

Selected Retina screenshots were visually inspected for legible controls,
selector wrapping, field direction, off/used cues and upgrade-panel clearance.

- [Compact native log](dro-38-native-640x480.txt)
- [Normal native log](dro-38-native-1120x720.txt)
- [Compact selector](../images/dro-38-640x480-environment-selector.png)
- [Compact ready fields/site](../images/dro-38-640x480-environment-ready.png)
- [Compact off cycle](../images/dro-38-640x480-cycling-field-off.png)
- [Compact consumed repair](../images/dro-38-640x480-repair-used.png)
- [Normal fields/site](../images/dro-38-1120x720-environment-ready.png)

## Human follow-up and limits

Fly both directions and across field boundaries, including Mobility and beam slow,
to assess handling/readability. Compare spending the one repair charge early with
returning while badly damaged in Environment pressure. Check whether the six/four
cycle produces useful route choices. No numeric balance or human acceptance is
claimed. Campaign introduction remains behind the deferred human gate; full
catalog combination validation is DRO-41. Substep boundary sampling is intentional;
it does not exactly split continuous trajectories at field edges.
