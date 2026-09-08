# Scout response and enemy flight

Follow-up to the flight-controls enhancement, requested for implementation in the
same workspace and `codex/drone-flight-controls` branch. This supersedes the
initial scout tuning and constant-speed chaser movement. The keyboard bindings
and physical player-flight behavior remain as specified in the flight-controls
design.

## Problem and intended result

Enemies currently start and redirect instantly at 150 world units/second. The
scout must first tilt and then accelerate at at most 60 world units/second squared
under neutral thrust. Raising its speed limit alone cannot fix the slow launch.

The scout should respond promptly, accelerate fast enough to evade from rest,
and have a clear advantage in speed and agility. Pursuers should visibly fly:
turn, pitch/bank, accelerate, brake, and retain momentum through direction changes.

## Scout tuning

| Setting | Previous | New starting value |
| --- | --- | --- |
| Maximum horizontal speed | 240 | 420 world units/second |
| Gravity / neutral rotor acceleration | 120 | 360 world units/second squared |
| Maximum total tilt | 30 | 30 degrees |
| Tilt response | 90 | 240 degrees/second |
| Leveling response | 120 | 300 degrees/second |
| Yaw response | 120 | 240 degrees/second |
| Boost thrust multiplier | 2 | 4/3 |
| Reduced thrust multiplier | 0.5 | 5/6 |

Raising gravity and matching neutral thrust triples horizontal acceleration
without breaking the rotor-force model. Adjusting the boost/reduced multipliers
preserves level-flight vertical acceleration at +120/-60 respectively. Tilt still
costs altitude, with no automatic compensation for the player. Drag, smooth speed
taper, independent leveling, collision constraints, and reset rules remain.

Response checks must demonstrate full tilt within 0.125 seconds, at least 65
horizontal units/second after half a second of forward input from rest, and at
least 140 after one second in an unobstructed arena. These are behavior targets,
not assertions that merely copy configured values.

## Pursuer flight

Use the same flight state and integration as the player, with an enemy profile
and an AI control source. The flight simulator accepts analog pitch/bank, yaw,
and rotor thrust plus the actor's collision envelope. The keyboard adapter keeps
its current cancellation/normalization semantics; AI tilt magnitude must remain
analog so the pilot can make small corrections and brake near its target.

Start pursuers stationary, level, facing toward the player's starting position.
Use a lower horizontal speed cap (260), tilt limit (20 degrees), tilt/leveling
rates (100/150 degrees per second), and yaw rate (120 degrees per second). Share
the world's gravity of 360 and drag behavior. The AI may select rotor thrust
between 0.5 and 2 times neutral to climb, descend, and compensate for its own
banking as a pilot would; this is input to the same physics, not direct movement.

The pilot steers toward the player's current position, without perfect prediction
or teleporting. Compute bounded desired velocity with arrival slowdown, then
velocity-error acceleration and a corresponding tilt/thrust request. Limit yaw
to the enemy profile. Recompute controls within bounded simulation substeps so
long frames do not cause stale steering. Opposite target changes require physical
braking/reorientation; world-space velocity never snaps to the new pursuit vector.
Pursuers must still reach a stationary target at different altitudes and boundaries.

Do not directly clamp enemies to the player's contact surface or overwrite their
velocity to match the player. Arrival control should reduce overshoot, but inertia
may carry an enemy past its target. Preserve the existing damage and projectile
rules, including first-hit-only swept projectiles and shared invulnerability.

Rotate enemy visuals with their flight attitude. Their existing orange cubes are
retained. Use a common rotated-envelope helper for arena bounds, contact damage,
and projectile collision. Retain previous enemy positions for projectile sweeps;
collision extents must conservatively account for enemy orientation over the
frame. No new assets, physics dependencies, waves, or combat balance changes.

## Validation and delivery

- Test the reported slow scout launch before tuning, then verify response targets.
- Test enemy acceleration from rest, capped speed/yaw/tilt, visible attitude,
  world momentum after abrupt target changes, braking/arrival, altitude pursuit,
  boundary containment, and approximate frame-rate independence.
- Add a default-encounter regression: immediate forward evasion should retain full
  hull for the first two seconds. This checks launch opportunity, not indefinite
  survival or final balance.
- Preserve projectile, contact, death, and complete restart regressions. New enemies
  start with zero momentum; death freezes both sides' flight state.
- Verify formatting, tests, Clippy, native build, and independent review. Update
  current controls/tuning/playtest documentation, commit, and push to update PR #5.
- Native visual checks should distinguish observable rendering/AI movement from
  held-key player feel that still needs a human playtest.
