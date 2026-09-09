# DRO-7 — Swarms and combat feedback

Source: https://linear.app/drone-survivors/issue/DRO-7

Direction approved during ticket refinement on September 8, 2026. Numeric
defaults below are provisional implementation and playtest starting points.

## Outcome and scope

Chunk 3 of 24 · Phase A — Prove tactical combat. Depends on completed
[DRO-6: Basic combat](https://linear.app/drone-survivors/issue/DRO-6).

Replace the fixed three-enemy encounter with a repeatable three-minute survival
scenario: timed bursts of flying chasers create escalating pressure, with brief
pauses in spawning to let the player reduce the remaining threat. Preserve the
current 3D arena, scout controls, physical enemy flight, straight automatic
projectiles, and shared contact-invulnerability rule.

The player should visibly defeat groups while maneuvering, distinguish hits,
kills, and incoming damage, and reach a clear encounter end without downstream
systems. Lulls stop spawning; they do not remove surviving enemies or guarantee
an empty arena.

## Encounter pacing and population

- Advance a run-local timer only while playing. End the survival scenario at
  180 seconds; no wave requires clearing before the schedule advances.

Use a repeatable authored schedule with three increasingly demanding phases.
Proposed initial warning times and batch sizes are below; enemies activate
after the warning duration:

| Phase | Warning times, seconds from run start | Enemies per burst |
| --- | --- | --- |
| Opening | 3, 15, 27, 39 | 3 |
| Pressure | 60, 70, 80, 90 | 4 |
| Final push | 120, 128, 136, 144, 152 | 5 |

- Start with a playable live-enemy cap of 30. This is independent of the
  150-enemy performance scenario. Tune batch sizes, intervals, cap, enemy health,
  and basic-weapon values together and record the final values.
- Count pending spawn reservations toward the cap so simultaneous warnings and
  deferred entity creation cannot overshoot it. Release reservations on spawn,
  cancellation, restart, or encounter end. Never remove living enemies to make room.
- Drop the portion of a burst that cannot fit; do not bank spawn debt. If a long
  frame crosses several burst times, process at most the latest due burst and
  discard older ones. Do not spawn after the encounter has ended.
- Current weapon tuning is four hits per kill at two shots per second: roughly
  30 kills/minute with perfect accuracy. Validate real kill throughput and
  survivability instead of treating the cap as a desired population at all times.

## Safe spawning and swarm movement

- Choose spawn positions near the arena perimeter, across multiple sides and
  altitudes. Use a fixed seed or deterministic candidate order for repeatability.
- Keep each full enemy body inside the flight volume. Candidates must not overlap
  other enemies or reserved spawn positions and must clear the player's full
  rotated collision bounds by a configurable margin, initially 120 world units.
- Show an in-world warning at the intended position for 0.75 seconds before
  activation. It cannot attack, take damage, or be targeted as a living enemy.
- Recheck bounds, occupancy, and player clearance when the warning completes.
  Cancel unsafe spawns; never relocate a warned spawn without a new warning.
  Candidate search is bounded, and no valid position means skipping that spawn.
- Add lightweight local separation through the enemy's steering inputs. Enemies
  still accelerate, bank, turn, and brake through the existing flight model.
  Separation should discourage persistent stacking without teleporting bodies,
  pushing the player, or introducing rigid-body collision/navigation systems.

## Combat feedback and HUD

- A successful nonlethal hit briefly flashes the affected enemy. A lethal hit
  removes the enemy once and emits a distinct short kill effect at its position.
- Applied player damage produces a brief hull/HUD cue. Indicate the existing
  0.75-second shared invulnerability period; blocked contacts do not replay the
  damage cue. Keep drone orientation and nearby threats readable.
- Use simple reusable visual assets. Cap simultaneous transient effects and
  expire them; dropping cosmetic effects under load must not drop damage or kills.
  Flashing one enemy must not flash every enemy sharing its base material.
- Show hull health, time remaining, living enemies, run kills, and current phase
  or spawning-lull status. Zero living enemies before 180 seconds is a lull,
  never an early victory or the old "ARENA CLEAR" state.
- Audio, floating damage numbers, camera shake, and polished particles are
  deferred; visual feedback must communicate the encounter by itself.

## End states and restart

- At 180 seconds, if alive, enter a stable survived state, freeze gameplay,
  cancel pending spawns, and show "SURVIVED" with kills and R to restart.
  Clearing remaining enemies is not required. This is a test-encounter result,
  not a campaign mission/reward flow.
- At zero hull, enter the existing dead state once and freeze gameplay. Define
  update precedence as restart, then fatal combat damage, then survival completion;
  fatal damage on the completion update results in death, not both outcomes.
- Neither terminal state allows further movement, damage, firing, spawning,
  scoring, or encounter-time progression. Cosmetic effects may finish expiring.
- R works during combat, lulls, pending warnings, death, and survival. Restore
  initial player position/attitude/momentum, full health, weapon readiness,
  invulnerability, zero elapsed time/kills, and the original schedule/seed. Remove
  all prior enemies, shots, warnings, reservations, and effects. Reset wins over
  held movement and combat on that update. Escape continues to quit.

## Implementation boundaries

Keep run timing, wave scheduling, reservations, and reset logic within focused
combat modules. Keep flight integration in the existing arena flight code and
presentation in the combat scene layer. Resolve hits, kills, and player damage
once in gameplay and publish their outcomes for effects and counting; presentation
must not own health or kill accounting. This also gives DRO-10 a later integration
point for XP without implementing XP now.

Group scenario and feedback tuning into explicit configuration alongside existing
combat tuning. Retain controlled-time headless ECS tests for gameplay rules.
Do not introduce an encounter editor, general event framework, or speculative
optimization architecture for this chunk.

## Acceptance and validation

- [ ] A full 180-second encounter progresses through timed bursts and lulls,
  supports visibly defeating groups, and ends once in survived or dead state.
- [ ] In ordinary evasive play, enemies remain individually readable and the
  player can identify enemy hits, kills, applied hull damage, and invulnerability.
- [ ] Live enemies plus reservations never exceed the configured cap, including
  simultaneous spawns, kills, cancellations, long frames, and repeated restarts.
- [ ] Unsafe/occupied spawn positions are skipped or cancelled with no hidden
  relocation, unbounded search, spawn debt, or post-end activation.
- [ ] Chasers pursue at different altitudes using physical flight; local
  separation reduces sustained stacking and preserves arena bounds.
- [ ] A kill increments the counter exactly once, including simultaneous lethal
  hits. Cosmetic limits and expiry cannot alter combat outcomes.
- [ ] HUD, timer, outcome precedence, frozen terminal states, and complete reset
  behave as specified, including reset on a spawn or completion update.
- [ ] Meaningful automated checks cover schedule boundaries, cap/reservations,
  spawn revalidation, hitch handling, kill accounting, feedback cleanup, terminal
  states, and reset. Retain existing movement and combat regression coverage.
- [ ] Run cargo fmt --check, cargo test, and cargo clippy --all-targets. Use
  cargo dev for native gameplay checks, including one full survival run and
  deliberate death/restart. Record human handling checks that remain unverified.
- [ ] Separately exercise a repeatable stress configuration with 150 simultaneous
  live enemies, active pursuit, projectiles, and feedback. A total of 150 spawned
  over time does not qualify. Document how the load is established and sustained;
  any harness-only changes must be explicit and excluded from gameplay claims.
- [ ] Record target Mac hardware, OS, commit/build profile, resolution, sampling
  duration, actual enemy counts, median/p95/p99 frame times, and hitch counts
  above 33.3 ms. Target 60 FPS, using p95 frame time at or below 16.7 ms as the
  provisional sustained-load goal. Separate warm-up/loading from measurement.
- [ ] If 150 fails the goal, record the failure and measured supported density;
  simplify effects or lower the playable density and repeat. Do not claim the
  150-enemy goal passed based on the reduced run. Performance and combat feel
  are separate acceptance evidence.

## Out of scope

New enemy roles, additional weapons, homing/piercing, knockback, obstacles,
energy/charging, modules, XP/upgrades, pickups/rewards, campaign progression,
mission hub/results flows, and changes to scout controls or its model.

## Definition of done

Acceptance checks pass, with any provisional performance fallback explicitly
recorded; the encounter works independently of unfinished downstream features
and has no known blocking defect. Record actual tuning, gameplay observations,
performance results, limitations, and next adjustments in docs/playtests.md.
Review and commit the implementation chunk separately.

Source plan: docs/superpowers/plans/2026-09-08-game-development-work-plan.md.
