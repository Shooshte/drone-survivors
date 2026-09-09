# DRO-9 — Four-slot module controls

Approved September 9, 2026. Implement on a separate branch; commit milestones,
push, and open a PR against main.

## Outcome and scope

Prove tactical power choices in the existing three-minute arena. Depends on
[DRO-8](https://linear.app/drone-survivors/issue/DRO-8/04-energy-and-charging).
Four interchangeable slots use keys 1–4, each initially equipped with a unique
prototype. Equipment remains fixed during a run; shop, ownership and player
loadout editing belong to DRO-16. No new dependencies, campaign or upgrade work.

## Modules and provisional tuning

| Default slot | Module | Effect | Continuous drain |
| --- | --- | --- | --- |
| 1 | Weapon overdrive | 2x basic weapon firing rate; no effect on rockets | 10/s |
| 2 | Shield | Blocks one complete direct-damage event; restores the block after 5 powered seconds | 8/s |
| 3 | Mobility boost | +25% horizontal thrust acceleration and maximum horizontal speed | 8/s |
| 4 | Rocket launcher | Automatic rocket every 2 seconds; 20 damage per enemy within 70 world units of impact | 10/s |

Shield capacity and recharge duration are definition-driven. Recharge advances
only while enabled and powered; progress pauses while off or depleted. A ready
shield also consumes energy while enabled. Blocked contact starts the existing
0.75-second shared contact-protection window. Switching never replenishes blocks
or resets recharge. Start/restart with one ready block. No hull healing.

Rockets target the nearest living enemy in range using 3D distance and stable
ties. Start with the basic weapon's 400-unit range, straight non-homing flight,
500-unit/second speed and 1.5-second lifetime. Explode on first enemy impact;
misses expire or leave the arena without an explosion. Radius is measured in 3D
from impact to enemy centers at impact time, inclusively; the directly struck
enemy is included and each enemy takes damage once. No player splash damage.
In-flight rockets finish after disabling. Cooldown continues with elapsed gameplay
time while off; toggling cannot bypass cooldown or bank a burst.

Mobility changes horizontal thrust acceleration and the speed cap only. Preserve
vertical thrust, gravity, drag, turn/tilt response, collision containment and enemy
flight. Disabling restores the normal cap on the next movement integration.

## Power, controls and lifecycle

- Full 100-energy battery on start/restart; all four modules off.
- Each key press toggles only its slot; holds do not repeat. Empty slots are safe.
  A module type cannot be duplicated; any type can occupy any of the four slots.
- Enabling requires at least 10 stored energy, with no activation fee. Disabling
  is always allowed. Simultaneous presses use the same stored battery threshold.
- Enabled modules drain continuously, including without targets, while shield
  is ready/recharging, or while the drone is stationary. Drain adds across slots.
- Preserve both charging fields and their geometry, non-stacking 25/s charging,
  and one combined net-rate clamp to [0, capacity]. All four drain 36/s: net -11/s
  even inside a charger. No outside regeneration.
- At zero, disable all modules and pause shield recharge. Charging does not
  reactivate modules. Basic automatic fire and every normal flight control remain
  available. Preserve progress when retiming the basic weapon cooldown.
- Restart wins over inputs and resets battery, module state, shield, rocket
  cooldown, projectiles and transient feedback. Death/survival freeze gameplay.
- Sample charging after movement. Movement uses the power state resolved at the
  previous gameplay update; module input/power is staged before contact damage and committed after
  contact/survival only if gameplay continues. Shield uses this staged state;
  new firing uses committed state. Terminal frames discard staged power changes.
  Integration tests cover activation, depletion/contact ordering and freeze.

## Presentation

Show all four slots with key, name, ON/OFF/EMPTY state and actual/configured drain.
Shield additionally shows READY or remaining recharge time (paused while off).
Show shared battery, total module drain, signed net charging, low-power rejection
per slot, and paused terminal state. Preserve readability at 640x480. Distinct
rocket visuals and a short impact-radius effect make splash understandable.

## Acceptance checks

- [ ] Exercise every slot independently and all four together: correct effect,
  drain, activation, held input, deactivation and depletion.
- [ ] Exercise every module in each of the four positions; empty slots are safe;
  duplicate module definitions are rejected.
- [ ] Verify additive drain, charging net rate, capacity/zero boundaries, low
  energy rejection and manual reactivation across frame rates.
- [ ] Shield blocks exactly one eligible hit, preserves hull, grants contact
  protection, recharges only while powered, and cannot be refreshed by toggling.
- [ ] Mobility improves horizontal acceleration/speed without changing vertical
  motion, ordinary controls, collision rules or enemy flight.
- [ ] Rocket impact damages multiple enemies once each in 3D radius, includes the
  direct target, excludes out-of-radius enemies, handles moving targets and
  misses, preserves launched rockets and cannot exploit toggles for bonus shots.
- [ ] Depletion removes powered effects; restart and terminal states are correct.
- [ ] Compare maximum-output and conservation in the same encounter and record
  power decisions, hull, kills, energy and limitations in docs/playtests.md.
- [ ] Run cargo fmt --check, cargo test --locked, cargo clippy --all-targets
  --locked -- -D warnings; use cargo dev for native validation.
- [ ] Review, commit milestones, push and open PR against main.
