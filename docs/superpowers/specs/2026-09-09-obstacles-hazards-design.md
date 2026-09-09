# DRO-11 — Obstacles, hazards, and route choices

Approved for implementation September 9, 2026, on a separate worktree/branch; commit milestones, push, and open a PR against main.

## Outcome and scope

Chunk 7 of 24 in the Drone Survivors development work plan.  
**Phase A — Prove tactical combat**

Make terrain create a readable choice when travelling between the two existing charging fields: take a longer route without environmental damage, or time a shorter hazardous passage while enemies pursue.

**Depends on:** [DRO-8: Energy and charging](<https://linear.app/drone-survivors/issue/DRO-8/04-energy-and-charging>), [DRO-9: Four-slot module controls](<https://linear.app/drone-survivors/issue/DRO-9/05-four-slot-module-controls>).  
**Unblocks:** [DRO-12: Combat prototype gate](<https://linear.app/drone-survivors/issue/DRO-12/08-combat-prototype-gate>).

Keep this independently demonstrable in the existing three-minute arena. The five-minute combined prototype and temporary upgrades belong to their respective tickets.

## Confirmed direction

Confirmed during refinement on September 9, 2026: mix low cover with a tall divider, and keep the shortcut hazardous at every altitude.

* Low cover can be flown over when the whole drone clears it.
* The divider spans floor to ceiling so climbing cannot bypass the intended route choice.
* Its shorter passage contains one non-solid hazard volume spanning floor to ceiling.
* A longer route around the divider remains outside that hazard. “Safe” means safe from environmental damage; enemies can pursue on either route.
* Both routes connect the same sides of the arena, work in both directions, and remain traversable using ordinary flight with an empty battery.
* Keep the starting position and both existing charging volumes clear. Provide room to turn, brake, and wait outside the hazard. Do not place a charger inside the hazard.

## Obstacle and weapon rules

* Use reusable, static, axis-aligned solid pieces with configurable position and dimensions. Start with low blocks and tall wall segments.
* Player and enemy collision accounts for the whole rotated body. Contact removes velocity into the surface while preserving sliding and movement away; touching a wall causes no damage.
* Prevent penetration and tunnelling, including mobility-boosted flight, corner contacts, climbing over low cover, and turning or tilting beside a wall. Resolve movement through the existing flight model.
* Solid terrain blocks basic projectiles and rockets. Resolve the earliest obstacle or enemy impact along the shot's travel; terrain wins a tie.
* Both automatic weapons select the nearest living target in range with an unobstructed firing line, retaining stable ties and existing cooldown rules. No visible target means no new shot.
* Proposed rocket rule: hitting terrain detonates the rocket at the impact. Preserve its existing radius, damage, and lack of player splash damage; solid cover also blocks explosion damage to enemies behind it.

## One telegraphed hazard

Use a cycling electrical field with visible emitters and a clear 3D boundary. Its volume must read as floor-to-ceiling, not as a floor-only effect.

**Proposed starting rules and tuning, to validate in playtests:**

* Repeat 3 seconds inactive → 1 second warning → 1 second active. Start/restart in the inactive phase. Inactive and warning phases deal no damage.
* Communicate all three states using motion/pattern or text as well as color. The warning must be visible before entering, and tall geometry must not hide the player or the passage.
* During an active window, body overlap or crossing the field can deal 10 damage to the player or an enemy. Each actor can receive at most one damage event or shield block per active window; entering late or leaving and re-entering does not grant another hit after that event.
* The player's powered, ready shield blocks one hazard event and consumes its block normally. Hazard and enemy contact share the existing 0.75-second protection window, so simultaneous sources cannot stack hits or consume multiple blocks. A suppressed event may become eligible later in the active window if the actor remains exposed.
* Enemies take the same hazard damage and use the existing hit/death feedback and kill accounting once. This permits luring a pursuer into the field.
* Ordinary flight can cross during an inactive window. Mobility can reduce exposure and a shield can protect a crossing; neither module is required.
* Hazard state uses gameplay time, freezes with gameplay, and resets on R. A hitch must not create accumulated burst damage or silently skip the warning before applying damage.

## Enemy routing and spawning

* For this authored layout, use a small set of clearance-checked waypoints/connections plus local steering. Keep the existing thrust, momentum, braking, and separation behavior.
* Route around the tall divider and around or over low cover. Connections must fit the enemy body through turns, not only its center point.
* Re-evaluate routes as the player changes sides; prevent indefinite wall pushing, corner oscillation, or route switching without progress. No teleporting or passing through terrain to recover.
* For this first version, enemies choose geometrically traversable routes without predicting the hazard cycle; the field can damage them on a shortcut.
* Validate enemy clearance and a navigable connection when creating a spawn warning and again before spawning. Reject positions inside solids or the hazard volume, while preserving existing player clearance, warnings, and population limits.

## Implementation boundaries

Keep world layout, obstacle queries, and hazard definitions in a focused world module; flight, combat, spawning, and presentation consume the relevant queries/events. Use typed Rust definitions and reusable placeholder visuals.

No general navigation mesh, physics-engine migration, procedural maps, moving/destructible obstacles, additional hazard types, new modules, pickups, campaign work, or map editor. Preserve existing flight controls, charging rates, module costs, and encounter duration.

The original planning allowance is four sessions. Reassess it after the collision/routing fixture works; simplify the layout before expanding navigation infrastructure.

## Acceptance checks

- [ ] A low block can be cleared with sufficient altitude; the tall divider and full-height hazard cannot be bypassed vertically. Visual and collision boundaries agree.
- [ ] Player and enemies cannot penetrate solids at faces, corners, or during rotations. Verify ordinary and mobility-boosted movement at 30/60/144 FPS and a 100 ms movement hitch.
- [ ] An isolated pursuer reaches a stationary player from either side using intended connections within 15 seconds; changing the target side produces a new valid route. At the normal 30-enemy cap, no living pursuer remains pinned to a corner or wall without route progress for 5 seconds in the route-change scenario.
- [ ] Basic shots, rockets, target selection, and explosion occlusion respect low cover and the divider; an obstacle before an enemy blocks the hit.
- [ ] Inactive/warning phases are harmless; active-phase entry, brief crossings, repeated entry, overlapping contact, shield readiness, depletion, and phase boundaries obey the damage rules without frame-rate-dependent extra hits.
- [ ] Start/restart and all authored spawn locations have valid clearance. Repeated R restores a consistent layout and hazard cycle without duplicating world entities; death/survival freezes hazard activity.
- [ ] With modules off, complete each route in both directions without environmental damage by timing the shortcut. Record representative travel times: the direct passage is shorter when open, while waiting for its cycle can make the detour preferable.
- [ ] During combat, demonstrate both routes and at least one deliberate timing, shield, mobility, or enemy-luring decision. Record why the choice helped; tune if one route is consistently preferable.
- [ ] At 640 × 480 and the normal window size, players can distinguish low cover, the tall divider, both routes, and the hazard warning without losing sight of their drone.

## Shared context

Build a roughly 90-minute, 12-mission campaign where tactical decisions make the player feel clever and combat progression makes them feel powerful. Use Rust/Bevy and the existing Cargo setup. Initial target: native macOS with keyboard/mouse. Use reusable content and placeholders until the full loop is proven. Numeric balance values are playtest starting points.

## Definition of done

- [ ] Acceptance checks above pass and the deliverable can be demonstrated independently of unfinished downstream features.
- [ ] No known blocking defect in the changed behavior; record scenarios, observed results, route timings, tuning changes, and next steps in `docs/playtests.md`.
- [ ] For code changes, run `cargo fmt --check`, `cargo test`, and `cargo clippy --all-targets`; use `cargo dev` for manual gameplay checks. Test meaningful rules and edge cases, not copies of static content.
- [ ] Review and commit this chunk separately.

Source: `docs/superpowers/plans/2026-09-08-game-development-work-plan.md` (September 8, 2026). Refined against the current 3D flight, charging, and four-slot module implementation on September 9, 2026.
