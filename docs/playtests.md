# Playtests

## Scout drone mesh — 2026-09-08

The scout replaces the combat player's primitive placeholder. Native Blender
5.2.1 generated the editable source, self-contained GLB and studio preview.
The export contains six material meshes, 39,816 triangles and no textures,
with a file size of 1,279,644 bytes. The gameplay collider remains unchanged.

Final checks: `cargo test --locked` passed all 32 tests;
`cargo clippy --all-targets --locked -- -D warnings`, `cargo fmt --check` and
`git diff --check` passed. Independent read-only review found no actionable defects.

Validation covers the actual GLB's transformed vertex bounds and normals,
loading through Bevy's GltfPlugin, parenting all six mesh entities beneath the
player, and preserving entities/material/mesh assets across three encounter
restarts. The pre-export Blender bounds assertion provides a second coordinate
check. The initial geometry test correctly failed when the GLB was absent.

Native visual inspection used the actual arena modules with temporary automatic
screenshot instrumentation. The full arena rendered the scout during combat,
with three hostiles and hull damage visible. A second capture moved only the
inspection camera, paused combat and hid enemies to examine the model's imported
surfaces and materials without obstruction. The capture harness is not shipped;
the normal game camera and combat behavior are unchanged. Bevy emitted its known
window-destroyed warning during clean exit.

For a manual check, run `cargo dev`, look for the silver body and cyan hover
rings, move in all three axes, approach arena boundaries, and press R repeatedly.
The complete scout should follow the player and remain present after each reset.
The studio render is in `docs/images/scout-drone.png`; native screenshots are
`docs/images/scout-arena.png` and `docs/images/scout-bevy-closeup.png`.


## DRO-6 — Basic combat — 2026-09-08

Target: native macOS / Apple Silicon, Rust nightly, Bevy 0.19.1.

### Repeatable combat smoke check

1. Run `cargo dev`. The drone starts at (0, 90, 0), with 100 hull and three
   orange chasers at fixed positions and different altitudes. Yellow shots fire
   automatically at the nearest enemy within 400 units of true 3D distance.
2. Move with WASD/arrows, Space, and Shift. Chasers follow in all three axes.
   Try the floor, ceiling, and side walls: neither body leaves the flight volume,
   and chasers can still make contact. Release input to hover.
3. Watch a shot after its target changes direction. It keeps travelling straight,
   can miss, and disappears after the first enemy hit, one second, or arena exit.
   Kill one enemy and confirm subsequent shots choose a remaining target.
4. Allow contact. Hull drops by 25, with at least 0.75 seconds before another
   hit even when several enemies overlap. Move away to avoid further damage.
5. Stand still until hull reaches zero. The destroyed prompt appears and all
   gameplay freezes. R restores full health, all three original enemy positions,
   starting altitude, and fresh weapon/damage timers. Repeat during combat;
   no previous enemies or shots should remain. Escape exits.
6. Evade long enough to kill all enemies. The clear prompt appears; movement
   remains available and R replays the same encounter. There are no waves,
   rewards, energy costs, or upgrade dependencies.

### Initial tuning

All values are grouped in `CombatConfig` in `src/combat.rs`:

| Setting | Starting value |
|---|---|
| Player / enemy health | 100 / 40 |
| Contact / projectile damage | 25 / 10 |
| Shared contact invulnerability | 0.75 seconds |
| Chaser / projectile speed | 150 / 650 world units per second |
| Fire interval / targeting range | 0.5 seconds / 400 world units |
| Projectile lifetime | 1 second |
| Chaser half-size / projectile radius | 14 / 3 world units |

These are playtest starting points. In particular, the user accepted the
invulnerability window provisionally and wants to judge it through play.

### Automated results

- `cargo fmt --check`: passed.
- `cargo test --locked`: all 31 tests passed (13 arena, 18 combat/presentation).
- `cargo clippy --all-targets --locked -- -D warnings`: passed.
- `cargo build --locked --features bevy/dynamic_linking`: passed.
- Real ECS tests cover three-axis chase and boundary contact, nearest/range/tie
  targeting, 30/120-update firing cadence, straight shots and moving-target
  sweeps, first-impact-only damage, lifetime/arena cleanup, shared contact timing,
  separation, lethal-hit ordering, frozen death, and repeated complete resets.
- Presentation checks verify one persistent HUD and no accumulation of live
  entities, meshes, or materials over repeated death/restart cycles.
- The initial test-first run produced 14 expected combat failures. Independent
  review then found target loss cancelling an active firing cooldown. A new
  regression reproduced the early extra shot; preserving the outstanding
  cooldown fixed it. Review found no other actionable issues.

### Native results and limits

`cargo dev` compiled and launched its process. For UI inspection, the same
rebuilt development binary ran in a temporary macOS app bundle with its Rust
standard-library lookup path added; no packaging changes are committed.

Observed the complete arena, orange chasers converging on the drone, the hull
counter falling in 25-point increments, an enemy kill (hostile count 3 to 2),
and the stable zero-hull destroyed state. Pressing R visibly restored 100 hull
and all three chasers at their initial positions. Escape terminated the app.
A missing dash glyph was found in the HUD during this check, replaced with an
ASCII separator, rebuilt, and visually verified in both active and dead states.

The UI tool sends brief keyboard taps. Held-key movement was not visibly
established in this smoke check; movement and altitude pursuit are covered by
the automated ECS checks. A full evasive native clear, deliberate missed-shot
scenario, projectile readability during movement, and human combat feel remain
manual playtest items. No final balance judgment is claimed.

### Next adjustment

Play the encounter with held controls and tune chase pressure, shot readability,
fire rate, and the 0.75-second damage window from observations. Ground enemies
with ranged attacks are deferred. Waves and richer feedback belong to DRO-7.

## 3D drone movement and bounded flight — 2026-09-08

Target: native macOS / Apple Silicon, Rust nightly, Bevy 0.19.1.

### Repeatable control smoke check

1. Run `cargo dev`. Confirm the mesh drone hovers over a gridded floor, with an
   open frame showing the sides and ceiling, a center marker, and the controls.
2. Use WASD and arrows to move over the X/Z plane. Space ascends; either Shift
   descends. Release input to hover. Try horizontal/vertical combinations: total
   speed stays constant. Opposing inputs cancel and duplicate bindings add no speed.
3. Hold Shift until the drone's bottom reaches the floor, then keep holding it.
   It must stop without sinking. Repeat with Space at the ceiling: the top stops
   at Y = 300. Add horizontal input while pressing into either surface, then move
   away from it. There must be no sticking, bounce, or movement through a limit.
4. Visit all side walls and corners, including while ascending/descending. The
   complete drone must remain inside the volume. The ground ring and vertical
   guide follow its X/Z position, making altitude visible.
5. Move in all three axes and press R. The drone returns to (0, 90, 0), and the
   ground ring returns to the center. Reset wins that frame; holding R does not
   repeatedly reset. Escape exits.
6. Resize wider, taller, and down to the minimum window size. The full flight
   volume and controls should remain visible. Relaunch restores the same scene.

### Automated results

- `cargo fmt --check`: passed.
- `cargo test --locked`: all 13 tests passed.
- `cargo clippy --all-targets --locked -- -D warnings`: passed.
- `cargo build --locked --features bevy/dynamic_linking`: passed.
- The movement tests drive the real gameplay plugin with keyboard resources and
  controlled time: all six directions/aliases, every two/three-axis diagonal,
  duplicate/opposing/released inputs, 30/120-update frame-rate equivalence,
  all 26 face/edge/corner directions across repeated ten-second frames, sliding
  along and leaving the floor/ceiling/side wall, and full-transform reset behavior.
- Scene tests instantiate the real scene builder and check every drone mesh's
  bounds against the movement half-extents, and every flight-volume corner
  against the camera projection at 1120 × 720, 640 × 480, 1600 × 480, and 640 × 1000.
- Test-first movement run: ten expected failures against the old 2D controller.
  A geometry regression test then caught a nose detail extending 0.1 units above
  the upper bound; repositioning the nose inside the bounds fixed it.

### Native results and limits

The development binary rendered successfully in a temporary macOS app bundle
using the same approach as the previous smoke check (only the development
library lookup path was adjusted; no packaging changes are committed).

The 3D drone, floor grid, full boundary frame, ground ring/vertical guide, home
marker, and control legend rendered. Resizing from 1120 × 720 to approximately
700 × 480 retained the full volume and readable controls. Closing the window
terminated the process successfully; relaunch restored the initial scene.
Bevy logged a window-destroyed warning on close, with exit status 0.

This session's UI automation did not produce observable gameplay keyboard input,
including Escape, on either direct or normal app launch. Native held-key movement
and reset are therefore not claimed as visually verified. Their ECS behavior is
covered by the passing tests; a human keyboard feel check remains useful.

Independent read-only source review found no actionable correctness issues.

## DRO-5 — Playable shell and test arena — 2026-09-08

Target: native macOS, keyboard input, existing Rust nightly / Bevy 0.19.1 setup.

### Repeatable control smoke check

1. Run `cargo dev`. Confirm a titled window opens with a visible arena, grid,
   center marker, placeholder drone, and control legend.
2. Move with W, A, S, D, then each arrow key. Release the keys; the drone stops.
   Try all four diagonal combinations. Their travel speed should match a
   straight direction. Combining W and Up should not add speed or skew a diagonal.
3. Hold movement toward each edge and each corner. All four rotors must remain
   inside the boundary. Move back inward to confirm the drone is not stuck.
4. Move away, press R, and confirm the drone returns exactly to the center marker.
   Repeat at least three times. Try R while holding a movement key: reset wins
   that frame, and held movement resumes on the next frame. Holding R should not
   repeatedly reset; release and press it to reset again.
5. Resize the window wider and taller. The complete arena remains visible and
   the drone's world-space bounds do not change.
6. Press Escape and confirm the application closes cleanly. Relaunch to confirm
   the same starting scene.

### Automated coverage

Seven tests drive the real gameplay plugin with keyboard input and a controlled
clock, without a display or GPU. They cover both keyboard layouts, eight movement
directions, normalized diagonals, duplicate and opposing bindings, stopping on
release, equal travel over one second at 30 and 120 updates, every edge/corner
including a ten-second update, repeated reset without entity duplication, and
reset precedence/held-key behavior.

The test-first check produced six expected failures against the stationary shell;
movement then passed five tests with two reset failures; adding reset passed all
seven tests.

### Results

- `cargo fmt --check`: passed.
- `cargo test --locked`: all seven tests passed.
- `cargo clippy --all-targets --locked -- -D warnings`: passed with no warnings.
- `cargo dev`: compiled and ran on macOS 26.6.2 / Apple Silicon without a
  reported runtime error. The UI tool cannot target a raw Cargo executable, so
  native interaction used a temporary app bundle containing that same binary
  with only its development-library lookup path adjusted; no bundle files are
  part of the repository.
- Native visual check: camera, full arena boundary, grid, center marker, drone,
  and control legend rendered correctly. Brief D input moved the drone and R
  returned it to the marker. Resizing from 1120 × 720 to approximately 700 × 480
  kept the whole arena and controls visible. Escape terminated the process;
  relaunch restored the initial scene.
- The UI tool sends very short taps, so sustained movement, all keyboard
  directions, frame-rate equivalence, every edge/corner, and repeated reset are
  verified by the automated ECS tests above. A human held-key feel check remains
  useful before tuning movement speed.
- Independent source review found no actionable correctness issues.

No known blocking gameplay defect was found. Speed (240 world units/second) and
arena dimensions (960 × 540) are initial playtest values.

### Next step

DRO-6: add the first chasing enemy, basic automatic weapon, damage, death, and
restart lifecycle. This arena currently has no combat or transient run entities;
reset restores the existing drone instead of recreating the scene.
