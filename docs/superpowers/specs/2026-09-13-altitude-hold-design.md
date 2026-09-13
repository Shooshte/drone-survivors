# DRO-33 — Automatic altitude hold

The player holds the actual flight height whenever Space and Shift yield neutral
input, including opposing keys. Release stops vertical momentum on the next
movement update. Explicit vertical input immediately restores ascent/descent,
using the existing level-flight forces and drag regardless of tilt. Horizontal
forces, braking, yaw, banking, modifiers and enemy flight remain unchanged.

The user's instruction authorizes implementation after refinement when no
questions are needed. The Linear ticket has been refined with these decisions.

## Approach

Use ideal altitude assistance in the keyboard adapter. A vertical-control enum
separates raw rotor physics (AI), assisted altitude hold, and tilt-compensated
manual thrust. The shared integrator skips vertical displacement and clears
vertical velocity in hold mode, then applies ordinary collision resolution.
The actual collision-resolved transform is therefore the hold target: no stored
height can become stale on spawn/restart or unreachable after rotation contact.
No position teleport or extra post-collision path is introduced.

Alternatives considered: a damped altitude controller preserves a little inertia
but adds overshoot/tuning and collision target state; multiplying total thrust by
inverse tilt cosine changes horizontal acceleration. Ideal vertical assistance
meets the requested ease of control while preserving horizontal handling.

## Acceptance and validation

- Collision-free neutral height drift <=0.002 units, zero vertical velocity after
  release, including consecutive height changes and cancelling inputs.
- Manual ascent/descent uses existing level acceleration and drag, including
  acceleration modifiers. Tilt does not affect its vertical trajectory.
- Collision sweeps and rotated bounds remain authoritative; after clearance
  correction the player stays at that reachable height and can depart normally.
- Real spawn/restart transform determines initial height, never a fixed target.
- Test production keyboard/movement paths at 30/60/120/144 Hz, mobility and heavy
  builds, long frames, obstacle top/underside/sides, and arena ground/ceiling.
- Update README/control help; run fmt, full tests and Clippy; record native flight
  observations in docs/playtests.md, distinguishing UI automation from human feel.

No new dependencies, enemy tuning, encounter tuning, or horizontal handling changes.
