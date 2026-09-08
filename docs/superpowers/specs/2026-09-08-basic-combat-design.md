# DRO-6 basic combat

Source: https://linear.app/drone-survivors/issue/DRO-6/02-basic-combat

Approved for implementation after ticket refinement on September 8, 2026.
Build on the existing 3D arena, preserving all flight controls and bounds.
One flying chaser type pursues in three dimensions. The drone automatically
fires visible straight projectiles at the nearest living enemy within true 3D
range, aiming at its current position without prediction or homing. A shot
damages the first enemy crossed and disappears. Misses expire or leave the arena.

Contact causes a hit followed by 0.75 seconds of shared player invulnerability;
this duration is provisional pending human playtesting. Dead enemies disappear.
Zero player health enters a stable dead state, freezes gameplay, and shows a
restart prompt. R resets the entire encounter, alive or dead, including health,
altitude, weapon timing, invulnerability, enemies, and projectiles. Reset wins
over movement and combat that frame. Escape still quits.

Use three fixed chasers at different positions/altitudes, with no waves or
automatic respawn. Clearing the encounter leaves movement and restart available.
Provide player health, enemy count, and readable dead/cleared status. Reuse
placeholder meshes/materials. No ground enemies, ranged enemies, separation,
knockback, obstacles, energy, upgrades, rewards, or mission lifecycle.

## Implementation boundaries

Use a small shared run state and ordered system sets. Arena movement remains
owned by ArenaPlugin; CombatPlugin owns tuning, chasers, projectiles, health,
and restart cleanup; CombatScenePlugin owns reusable visual assets and the HUD.
Headless tests drive the real ECS systems with keyboard input and controlled time.
Use swept projectile collisions, including enemy motion during the frame, to
avoid skipping targets. Stable entity identity breaks equal-distance ties.

Keep all balance values in CombatConfig. Initial values: player HP 100, enemy
HP 40, contact damage 25, shot damage 10, chase speed 150 units/s, projectile
speed 650 units/s, fire interval 0.5 s, range 400 units, projectile lifetime 1 s,
enemy radius 14 units, projectile radius 3 units. Use the drone's existing box
for contact so ground and ceiling contact remain reachable. These values are
starting points, not final balance requirements.

## Acceptance and validation

Verify 3D chase and targeting, range/tie handling, immutable shot direction,
first-impact-only damage, misses/cleanup, shared invulnerability and separation,
one-time death, frozen gameplay, repeated full reset, and reset precedence.
Retain existing arena tests. Run cargo fmt --check, cargo test --locked,
cargo clippy --all-targets --locked -- -D warnings, and native cargo dev.
Record actual results and native interaction limits in docs/playtests.md.
Review the complete diff, commit, push, and open a PR against main.
