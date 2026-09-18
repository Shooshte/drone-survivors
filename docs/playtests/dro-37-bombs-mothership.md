# DRO-37 — Collision bombs and mothership spawning

September 18, 2026. Agent verification on native macOS. Human balance and
counterplay acceptance remain pending. All new content stays in the catalog arena.

## Rules and counterplay

Three added scenarios: Collision bomb, Mothership, Mixed ordnance. The mixed
scenario launches one bomb carrier, one mothership and one jammer. Ordinary
campaign wave tables, shop choices, upgrade offers and save schema are unchanged.

A 20-hull red spiked carrier uses production pursuit and swept contact. Avoid
contact or destroy it first. Contact consumes the carrier without kill/XP/loot
credit and attaches one bomb, with no immediate damage. A second carrier cannot
stack or refresh the attachment. Shield/invulnerability do not prevent attachment.
The three-active-second fuse starts at attachment, including on a hitch. The HUD
shows remaining seconds and a red badge follows the drone. Detonation consumes
the bomb once and attempts 25 damage through ordinary shields/invulnerability;
blocked explosions never retry. Choices pause the fuse. Reset, return, cleanup
and terminal outcomes clear it, including an unresolved fuse at round completion.

Repulsor is a catalog-only slot choice using standard toggles, activation threshold,
jam locks and depletion. It drains 8 energy/second and emits a ready pulse while
powered, then waits two active seconds before another. A pulse removes an attached
bomb without damage or rewards, winning a tie with fuse expiry. Toggling does not
reset cooldown; time in menus does not advance it. The compact module row shows
cooldown, and a cyan ring marks a pulse. Only bomb removal is implemented here:
full pushback behavior and Repair remain DRO-39. Campaign purchases and saved
module decoding reject Repulsor; it is not added to campaign module enumeration.

A white faceted mothership has 100 hull and a 120-unit horizontal speed cap. It
approaches to 350 units using ordinary flight. Its collision hull is the same
14-unit half-size as other enemies; the larger launch rings are warning decoration.
Every six active seconds it attempts one child, never catching up missed attempts
with a burst. A full 1.2-second warning marks the site, parent and HUD. Child kinds
cycle Chaser, Fast, Rammer, Slower, Jammer, Bomber; never Mothership. A deterministic
16-site ring at radius 110 uses shared arena, obstacle, hazard, navigation, player
and occupied-hull clearance checks at admission and delivery. Pending warnings
reserve space in the same 30-enemy cap as authored waves and other parents. Rejected
attempts still advance the cycle. Killing a parent immediately cancels its pending
warning and releases its reservation. Existing children survive parent death.

## Automated verification

Production-system tests cover fuse timing at 30/60/144 Hz, one attachment on a
hitch, no refresh, shield/invulnerability, powered/empty/jammed/depleted Repulsor,
toggle cooldown, pause/reset/terminal cleanup, typed kill accounting and terrain
occlusion. Full combat-order checks cover lethal bombs suppressing launches and
lethal projectiles canceling parent delivery at 30/60/144 Hz.

Mothership tests exercise full warning duration at all three rates, the six-child
cycle, multiple parents sharing cap with authored waves, bounded hitch behavior,
admission and activation clearance, early/deadline source death, released capacity,
child survival, pause guard and terminal cancellation. Catalog tests select all
three scenarios, equip Repulsor and verify clean return to selection.

The campaign presentation regression caught hidden player cue meshes being
created outside the catalog. Those cues are now catalog-only. Source-death tests
and the native fixture also caught pending launches lingering until their deadline;
parent-linked warnings now cancel immediately and release capacity in the same frame.

Final checks:

- `cargo fmt --check` and `git diff --check`: pass.
- `cargo test --locked`: **447 passed, 5 existing probes ignored, 0 failed**.
  [Full test output](dro-37-tests.txt).
- `cargo clippy --locked --all-targets -- -D warnings`: pass.
  [Clippy output](dro-37-clippy.txt).
- Independent read-only code review found no actionable correctness or regression
  findings after inspecting ordering, power, cleanup, caps and campaign isolation.

Builds reused dependencies through
`CARGO_TARGET_DIR=/Users/shooshte/projects/drone-survivors/target`; source edits
and commits stayed in the separate DRO-37 worktree.

## Native checks

```sh
DRONE_ORDNANCE_SMOKE=1 DRONE_CAPTURE_DIR=/tmp/ordnance cargo dev --locked -- --validate catalog
DRONE_ORDNANCE_SMOKE=1 DRONE_CAPTURE_MINIMUM=1 DRONE_CAPTURE_DIR=/tmp/ordnance-small cargo dev --locked -- --validate catalog
```

The opt-in fixture uses synthetic selection/toggle keys, suppresses basic auto-fire,
and repositions each standalone carrier/parent after normal spawning. It fires one
synthetic 100-damage projectile at a parent while its second child is pending.
The mixed enemies are repositioned once to expose simultaneous bomb/jam HUD states.
It grants no health, energy, XP, kill counters or result. It asserts damage, removal,
restart, child activation, parent death, cancellation and scenario roster. Text bounds
are checked at captures; navigation/HUD clearance is checked every gameplay frame.
This is rule/presentation evidence, not a difficulty or human playtest.

Selected screenshots show the selector, bomb countdown, dislodging, launch warning
and simultaneous bomb/jammer lock. A screenshot pass corrected a repeated cooldown
label and an unsupported dash glyph. Existing native winit shutdown warnings can
appear after PASS; they do not indicate a failed fixture.

Both final fixtures passed at **1120×720** and **640×480**, including simultaneous
bomb/jammer HUD states. Selected Retina captures were visually inspected for
countdown, readable copy, silhouette/phase cues and control clearance.

- [Normal native log](dro-37-native-1120x720.txt)
- [Compact native log](dro-37-native-640x480.txt)
- [Compact selector](../images/dro-37-640x480-bomb-selector.png)
- [Compact bomb countdown](../images/dro-37-640x480-bomb-countdown.png)
- [Compact dislodged bomb](../images/dro-37-640x480-bomb-dislodged.png)
- [Compact mothership warning](../images/dro-37-640x480-mothership-warning.png)
- [Compact simultaneous bomb/jammer lock](../images/dro-37-640x480-ordnance-mixed-lock.png)
- [Normal mothership warning](../images/dro-37-1120x720-mothership-warning.png)

## Human follow-up

- Fly the carrier scenario with empty slots, Shield and Repulsor; assess whether
  avoiding contact and recognizing the three-second fuse feel fair.
- Test the Repulsor at low battery and during a jammer lock. Compare enabling it
  reactively with the ongoing cost of leaving it enabled.
- Destroy a mothership during its warning, evade its children and use cover/altitude.
  Check whether the six-second cadence and 100 hull create a useful target priority.
- Compare mixed ordnance with existing mixed pursuers/control. Numeric tuning is
  provisional, and this check does not complete DRO-41 combination validation.
- Keep campaign introduction behind the deferred human gate.
