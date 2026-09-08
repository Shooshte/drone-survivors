# Drone flight controls

Status: control scheme approved; written specification ready for review.
Scope: specification only. Implementation requires a subsequent user request.

## Purpose

Replace fixed-world, immediate movement with heading-relative flight driven by
rotor thrust, gravity, and momentum. The visible scout orientation must match
the direction of its thrust. Pitch and roll automatically level when released,
but neither velocity nor altitude is automatically restored.

This supersedes the movement behavior in the earlier 3D drone design. Existing
arena dimensions, starting position, camera, model asset, and combat rules remain.

## Agreed controls

| Input | Behavior while held | Behavior when released |
| --- | --- | --- |
| W / Up | Pitch nose down, accelerating forward relative to heading | Smoothly return pitch to level |
| S / Down | Pitch nose up, accelerating backward relative to heading | Smoothly return pitch to level |
| A / Left | Turn heading left around world vertical | Stop turning; retain heading |
| D / Right | Turn heading right around world vertical | Stop turning; retain heading |
| Q | Bank left, accelerating toward the drone's left | Smoothly return roll to level |
| E | Bank right, accelerating toward the drone's right | Smoothly return roll to level |
| Space | Boost rotor thrust above neutral | Return to neutral thrust |
| Either Shift | Reduce rotor thrust below neutral | Return to neutral thrust |
| R | Restart the encounter and all flight state | Existing restart behavior |
| Escape | Quit | Existing quit behavior |

W/S replace the previously discussed manual throttle binding. They control
pitch, not direct velocity. Space/Shift control thrust, not direct vertical
velocity or an altitude target. Throttle is not latched between key presses.

Duplicate aliases do not stack. Opposing inputs cancel independently: W+S and
Q+E request leveling on their respective axes; A+D stops yaw; Space+Shift uses
neutral thrust. Releasing one tilt axis does not cancel the other. Controls can
be combined, including turning while pitched or banked.

## Flight behavior

Rotor thrust acts along the drone's rotated local up axis; gravity acts downward
in world space. Neutral thrust equals gravity's magnitude. A stationary, level
drone therefore hovers. Pitching or banking redirects part of that thrust
horizontally and reduces its upward component. There is no automatic increase
in thrust to compensate for tilt, including when both tilt controls are held.
Space can compensate manually; it is not guaranteed to produce ascent at every
velocity. Shift can slow an ascent before producing descent.

Maintain world-space velocity and integrate acceleration over time. Leveling
removes the horizontal thrust component, but leaves existing momentum. Passive
drag gradually reduces drift; counter-pitching or counter-banking brakes faster.
Leveling after a descent does not restore lost altitude or instantly cancel
downward velocity. Boosting thrust can arrest that descent.

Yaw changes the heading and the horizontal direction of any tilted thrust.
It does not rotate existing world-space velocity. A level drone can turn in
place; a moving drone can face a new direction while still drifting along its
previous path. Model forward is local -Z and model up is local +Y.

Tilt grows progressively while held, up to a bounded angle. Combined pitch and
roll share a maximum total tilt away from world vertical, so diagonally tilting
cannot exceed that angle. Use the input direction in the heading's horizontal
frame to define the desired tilt; normalize combined pitch/roll input before
applying the shared limit. Returning to level is smooth and does not overshoot.
Full flips, inverted flight, and uncontrolled angular momentum are out of scope.

## Speed and proposed tuning defaults

Maximum speed means the magnitude of horizontal X/Z velocity, irrespective of
heading or input combination. Vertical velocity is excluded. Forward, backward,
sideways, and diagonal flight share this limit. Additional outward horizontal
acceleration tapers smoothly from 90% of maximum speed to zero at the limit;
braking and steering remain available. A final magnitude clamp prevents numeric
overshoot. This limiter never increases rotor thrust or compensates vertical lift.

These values are proposed starting points for playtesting, not additional user
requirements. Keep them together in a flight configuration resource.

| Parameter | Initial value |
| --- | --- |
| Maximum horizontal speed | 240 world units/second, matching the current movement speed |
| Maximum total tilt from vertical | 30 degrees |
| Tilt response rate while input is held | 90 degrees/second |
| Return-to-level rate | 120 degrees/second |
| Yaw rate | 120 degrees/second |
| Gravity magnitude | 120 world units/second squared |
| Neutral thrust acceleration | 1 times gravity |
| Space thrust acceleration | 2 times gravity |
| Shift thrust acceleration | 0.5 times gravity |
| Horizontal linear drag coefficient | 0.25 per second |
| Vertical linear drag coefficient | 0.5 per second |

Drag opposes velocity and cannot reverse it. Use stable time integration so long
frames do not reverse damping or overshoot leveling. There is no separate hard
vertical speed cap in this enhancement; gravity, thrust, drag, and arena contact
determine vertical motion.

## Arena, collision, and reset

The full rotated model must remain inside the flight volume. Derive world-axis
half-extents from its current orientation and the existing local collision
envelope (35, 15, 45). Use this conservative rotated bounding box consistently
for arena containment and player/enemy contact damage. Accurate mesh collision
and a general rigid-body physics engine are outside this enhancement.

At a wall, floor, or ceiling, correct position and remove only velocity directed
into that surface. Keep tangential motion and motion away from contact. Rotation
near a boundary may require a positional correction to keep the model contained.
Contact does not bounce, damage the drone, or change its attitude. A drone resting
on the floor can take off when thrust supplies sufficient upward acceleration.

R restores position (0, 90, 0), default heading toward -Z, level attitude, zero
linear velocity, and neutral thrust alongside the existing full combat restart.
Reset takes precedence over held flight controls on that frame. Death freezes
flight, including gravity and leveling; restart resumes it from the reset state.

## Integration and presentation

Keep flight state and simulation in ArenaPlugin. Introduce explicit flight state
for velocity, heading, and tilt, and separate input interpretation, attitude
update, force integration, and boundary handling into focused operations. Use
fixed simulation steps or bounded substeps; simulation must remain ordered before
combat, with reset first. Rendering reflects the simulation orientation directly.

ArenaScenePlugin retains the existing fixed angled camera and ground marker.
Update the altitude guide's lower endpoint using the rotated bounds. Update the
on-screen legend, README controls, and playtest instructions with pitch, yaw,
banking, thrust, drift, and altitude loss. No model asset changes are required.

Automatic weapons continue aiming at the nearest enemy independently of drone
heading. Enemy behavior, weapon balance, health, and encounter lifecycle retain
their existing rules, with the rotated player bounds used for contact detection.

## Acceptance checks for implementation

1. Every binding, alias, opposing pair, and combined input behaves as specified.
2. A/D visibly turn the nose in the expected direction and preserve heading on
   release. Turning while drifting never instantly redirects velocity.
3. W accelerates along heading, S backward, Q left, and E right, including after
   a 90-degree heading change. Tilt and thrust direction agree visually.
4. Holding tilt reaches its limit; releasing it returns smoothly to level.
   Combined pitch/roll stays within the shared total tilt limit.
5. From stationary level hover, neutral thrust preserves altitude. Pitching or
   banking at neutral thrust produces altitude loss. Leveling retains downward
   momentum; sufficient boost arrests descent.
6. Released tilt preserves horizontal drift, passive drag reduces it gradually,
   and opposite tilt brakes it. Space/Shift release restores neutral thrust.
7. Horizontal speed never exceeds the configured maximum, including combined
   tilt, yaw, and boosted thrust. The speed limiter preserves braking and turning.
8. At all arena surfaces and representative rotations, the full model remains
   contained. Contact permits sliding, departure, and takeoff without accumulated
   velocity into the surface. Rotated bounds also govern enemy contact.
9. Reset during movement or death clears all flight state and restarts combat;
   death freezes motion and attitude. Automatic aiming remains independent of yaw.
10. Controlled-time tests verify comparable trajectories at common frame rates
    and stable behavior across a long render frame. Run formatting, tests, and
    Clippy, then manually playtest orientation, handling, boundaries, and combat.

## Alternatives considered

- Visual-only tilt would retain the previous movement feel and provide no banking
  physics. The user chose thrust-driven banking with momentum and altitude loss.
- A full manual attitude/throttle scheme would retain bank on release and place
  rotor power on W/S. The user chose automatic leveling and W/S pitch instead.
- The selected assisted flight model retains physical thrust direction, gravity,
  and momentum while using automatic leveling, bounded tilt, and a horizontal
  speed limit to keep keyboard controls manageable.
