# DRO-35 — Fast pursuers and double-impact rammers

September 18, 2026. Agent verification on native macOS; human acceptance remains
pending. This completes the next unfinished catalog child, preserving its existing
approved design and partial implementation in `.worktrees/dro-35-enemy-variants`.

## Delivered behavior

Three catalog-only scenarios: Fast pursuer, Double-impact rammer and Mixed
pursuers. Type selection survives production spawn warnings and safety checks.
Existing scenarios and campaign waves continue to spawn ordinary chasers; no
campaign, shop, upgrade-offer or save-schema changes.

Fast pursuers use the existing rotor pilot with a 350u/s horizontal cap and 20 hull.
Cyan bodies with paired fins identify them. Rammers have 80 hull, a magenta body,
phase-colored ring, locked warning/charge line and two visible impact pips. They
warn for at least one active second within 280u and clear sight before charging
with a 420u/s cap. They cannot retarget a committed charge. Retreat lasts at least
1.2 active seconds and requires 180u separation before another full warning.

Approach, warning and retreat contact are harmless. Hull hits and shield blocks
spend an impact; invulnerability rejects an impact but still forces retreat. A
three-second missed charge retreats without spending budget. The second accepted
impact destroys the enemy without kill/XP/drop rewards. Player-caused kills keep
the existing typed outcome/drop path. Temporary state pauses with gameplay and
is removed on restart/return.

## Automated evidence

- `cargo fmt --check` and `git diff --check`: pass.
- `cargo test --locked`: **403 passed, 5 existing probes ignored, 0 failed**.
  [Full output](dro-35-tests.txt).
- `cargo clippy --all-targets --locked -- -D warnings`: pass.
  [Output](dro-35-clippy.txt).
- New checks cover type activation/campaign isolation; faster physical pursuit;
  two naturally separated impacts at 30/60/144 FPS; warnings, misses,
  invulnerability/shields, reward facts; swept contact/wall rejection; solids;
  pause/restart/terminal states; and restoring the correct material after flashes.
- Independent review found two retreat stalls. Both were reproduced with failing
  tests before fixing: cached destinations when followed by the player, and
  oversized navigation clearance at wall contact. Retreat now uses the actual
  rotated physical hull and refreshes an exhausted destination. A production-flight
  regression verifies recovery after following the rammer at 30/60/144 FPS.
  Re-review found no remaining blocking issue.

Builds used `CARGO_TARGET_DIR=/Users/shooshte/projects/drone-survivors/target` to
reuse dependencies; all source edits and commits stayed in the issue worktree.

## Native evidence

Run using the normal Cargo development alias:

```sh
DRONE_VARIANT_SMOKE=1 DRONE_CAPTURE_DIR=/tmp/variants cargo dev --locked -- --validate catalog
DRONE_VARIANT_SMOKE=1 DRONE_CAPTURE_MINIMUM=1 DRONE_CAPTURE_DIR=/tmp/variants-small cargo dev --locked -- --validate catalog
```

The fixture is installed only with `DRONE_VARIANT_SMOKE` in catalog mode. It uses
synthetic keyboard presses to select/launch all three new scenarios. It suppresses
auto-fire and moves each single enemy once, after production warning activation,
to 250u beside the stationary player. It does not assign damage, impacts, kills,
XP, phase changes or rewards. From that setup, production flight/contact systems
naturally produce both rammer impacts, ending at **80 hull, zero kills**, with
warning/charge/retreat/second-warning all observed. It checks fresh hull on restart
and all three types in the mixed round. This is a behavior/presentation fixture,
not a fair-play balance run or evidence of human counterplay.

Both logical window sizes pass, including visible text bounds. Screenshots are
Retina captures at twice the logical resolution. Selected screenshots were
visually inspected for selector wrapping, type silhouette, warning line,
phase ring and remaining pips.

- [1120×720 native log](dro-35-native-1120x720.txt)
- [640×480 native log](dro-35-native-640x480.txt)
- [Compact rammer selector](../images/dro-35-640x480-rammer-selector.png)
- [Compact warning](../images/dro-35-640x480-rammer-warning.png)
- [Compact retreat after the first impact](../images/dro-35-640x480-rammer-retreat.png)
- [Normal fast pursuer](../images/dro-35-1120x720-fast-flight.png)
- [Normal second rammer warning](../images/dro-35-1120x720-rammer-second-warning.png)

The native run may log Bevy's existing `Skipped event Destroyed for unknown winit
Window Id` warning during successful window shutdown; no gameplay assertion failed.

## Known limits and human checks

Tuning is provisional. Native evidence uses synthetic starting positions and
suppressed auto-fire; tests do not establish enjoyable difficulty. Human checks:

1. Launch each new scenario with ordinary auto-fire and several module loadouts.
   Compare cyan pursuit against orange chasers, banking, altitude changes and cover.
2. Dodge a yellow line; confirm the red charge follows its commitment. Try wall
   cover and following a blue retreat. Check a complete warning precedes rearming.
3. Use Shield; confirm one pip is spent per accepted block/hit and contact during
   warning/retreat causes no damage. Shoot a rammer before its second impact.
4. Check visual readability in a mixed encounter at both supported window sizes.

DRO-21 human acceptance and campaign introduction remain gated. Additional enemy
types belong to DRO-36/DRO-37; modules/environment/upgrades/combinations remain
with the other DRO-22 children.
