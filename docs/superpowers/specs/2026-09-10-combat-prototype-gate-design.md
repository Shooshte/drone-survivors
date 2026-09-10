# DRO-12 — Combat prototype gate

Status: approved for implementation September 10, 2026. Work on a new branch,
commit milestones, push, and open a PR against main. Numeric pacing and playtest
targets are provisional; this specification does not claim successful playtests.

Linear: https://linear.app/drone-survivors/issue/DRO-12/08-combat-prototype-gate

## Outcome and confirmed direction

Refine the existing arena into a five-minute survival encounter combining combat,
charging, module management, temporary upgrades, obstacles, and route choices.
Use the completed DRO-5 through DRO-11 baseline, including the merged XP and
terrain work. The local checkout used for refinement predates those merges;
implementation must start from current main containing both features.

Surviving should take a few attempts for a player who understands the controls.
Provide a forgiving opening, then make pressure increase rapidly and feel
exponential. For this scenario, difficulty escalation comes only from increasing
enemy numbers: larger bursts, shorter spawn intervals, and, if necessary and
validated, a higher population cap.

Enemy health, contact damage, speed, acceleration, turning, pursuit, separation,
navigation, targeting rules, and hazard interaction do not scale or otherwise
change for difficulty. Keep the existing flying chaser; add no enemy roles or
adaptive difficulty. Repeated attempts use the same authored wave schedule and
existing deterministic spawn/upgrade seeds, so learning can improve results.

## Arena and player systems

Keep the current arena, low cover, tall divider, full-height electrical shortcut,
safe detour, two charging fields, exploration pickup, four equipped modules, and
six single-purchase temporary upgrades. Preserve player controls, baseline combat
stats, energy and charging rates, module costs/effects, upgrade benefits/drawbacks,
and the hazard cycle. Normal player-selected upgrades still change player stats.

Limit geometry and presentation adjustments to demonstrated clearance or
readability problems. Keep both routes traversable with an empty battery. A
charger must not become a protected area; existing enemies can pursue there.
Do not introduce new maps, equipment, rewards, objectives, campaign flow, or a
tutorial system. Concise control and state labels may be improved where playtests
show confusion.

## Encounter pacing — proposed starting schedule

Survival requires 300 seconds of active gameplay; upgrade-selection pauses do not
count. The first 45 seconds give the player room to orient, encounter enemies,
earn an early choice, and experiment with charging. Pressure begins accelerating
at 45 seconds instead of waiting until the last minute.

| Gameplay window | Burst size | Burst interval | Purpose |
| --- | --- | --- | --- |
| 0–45 seconds | 3 | 15 seconds; first warning at 3 seconds | Forgiving opening |
| 45–105 seconds | 3 | 12 seconds | Establish sustained pressure |
| 105–165 seconds | 4 | 8 seconds | Double the preceding active arrival rate |
| 165–225 seconds | 6 | 6 seconds | Double pressure again |
| 225–300 seconds | 8 | 4 seconds | Final survival test |

Window starts are inclusive and ends exclusive. Each pressure window begins with
a warning at its starting time and repeats at the stated interval. Suppress
warnings during the final six seconds of each pressure window, creating brief
spawning lulls. The opening warnings occur at 3, 18, and 33 seconds. These rules
produce one unambiguous, repeatable list of warning times; no burst is duplicated
at a window boundary. Enemies activate after the existing 0.75-second warning.

The active arrival rates after the opening are 15, 30, 60, and 120 enemies per
minute before lulls and safety/cap rejection. These are tuning inputs, not
promises of actual population or mathematical guarantees of difficulty. Lulls
stop new warnings; existing warnings can finish and living enemies remain.

Keep the existing cap of 30 living enemies plus spawn reservations for the first
playtest. Track requested, admitted, and rejected spawns and actual living counts.
If the cap or valid spawn positions flatten the final difficulty ramp, adjust
wave timing or validate a higher cap. Never substitute stronger or faster enemies
to compensate. Preserve safe spawning, terrain/navigation clearance, warnings,
bounded candidate search, and cancellation of unsafe spawns. Rejected spawns are
discarded, not banked. A hitch follows the existing latest-due-burst rule and must
not release accumulated waves.

Final burst sizes, intervals, cap, and lulls may change during tuning. Retain the
short forgiving opening and rapidly accelerating number pressure, and record the
actual final schedule. An early population plateau or an empty final stretch
fails the intended pacing even if the timer reaches five minutes.

## XP, choices, and lifecycle

Initially retain 10 XP per confirmed kill, 30 from the one-use exploration pickup,
and thresholds of 50, 75, 100, and so on. More kills naturally accelerate XP.
Target the first earned choice within roughly 30–60 active seconds and several
meaningful choices before the final pressure phase. Record actual timings; do
not guarantee a choice when the player avoids kills and the pickup.

XP rewards/thresholds may be tuned if the denser schedule exhausts meaningful
choices too early or interrupts play too frequently. Keep the six-upgrade catalog,
single-purchase eligibility, explicit Skip, stable offers, queued choices, and
nonblocking exhausted-pool behavior. Do not force selections, refill stats, or
grant synthetic XP in normal gameplay.

Selection freezes all gameplay, including waves, hazard phases, flight, shots,
energy, charging, cooldowns, and protection timers. Resume must not catch up wall
time. Preserve guarded mouse/keyboard selection and module-toggle isolation.

At 300 active seconds, an alive player survives without clearing remaining
enemies. Preserve precedence: restart first, fatal damage before survival, and
terminal outcomes before opening a choice. Death/survival freezes gameplay;
pending warnings are cancelled. R resets the complete run, including flight,
hull, energy, module/shield state, shots, enemies, warnings, effects, hazard cycle,
XP, choices, selected upgrades, pickup, elapsed time, and deterministic seeds.

## Playtest procedure and pass criteria

Begin with a baseline playtest of the current integrated arena before tuning.
Record existing friction rather than assuming the individual prerequisite tickets
proved the combined experience. Previous notes leave human checks outstanding
for sustained route use, shield/mobility timing, enemy luring, and dense-combat HUD
and hazard readability.

Observe two unfamiliar players if available. Otherwise record and review two
fresh human playthroughs, explicitly identifying the same-player fallback and
its limits for judging newcomer difficulty. Scripted pilots can check mechanics
and performance but cannot establish perceived difficulty, learning, or fun.

Give players the controls without prescribing tactics. Record attempts, active
death/survival times, kills, upgrade offer/selection times, selected upgrades,
module use, charging visits, route decisions, peak enemies, and reported causes
of failure. A provisional target is a first success after approximately 2–5
attempts by a controls-familiar newcomer; a small playtest sample is directional
evidence, not a guaranteed completion rate. Record failed attempts as well as wins.

The gate passes when:

- The opening permits orientation, while pressure clearly accelerates soon after
  it and continues to challenge an upgraded build through the final minute.
- Two meaningfully different tactics each complete a normal five-minute run.
  Candidate contrasts are mobile energy conservation and heavier powered offense;
  observed successful alternatives are acceptable. Different upgrade names alone
  do not establish different tactics: record differences in power or route use.
- Players can explain one useful charging/module decision, identify a specific
  increase in combat strength, and describe an upgrade drawback that changed a
  decision. Record concrete examples rather than inferring understanding from wins.
- The full-height hazard warning, drone orientation, health/energy, module state,
  and choice cards remain readable at the normal window size and 640 × 480.
- No blocking collision, route, input, pause, reset, or outcome defect remains.
  Fix the highest-impact observed issues and repeat affected scenarios.

If fresh-player evidence is unavailable, state that limit and keep newcomer
difficulty unverified. Missing required human evidence leaves the gate pending;
automated victories or a clean build alone do not close DRO-12.

## Implementation and validation boundaries

Keep authored pacing in the existing wave configuration, XP adjustments in the
existing upgrade configuration/accounting, and presentation in the scene layer.
Reuse existing gameplay, world queries, validation tools, and lifecycle systems.
Add only the observation data needed to diagnose population pressure and choices;
no analytics service, generalized encounter editor, or navigation rewrite.

Meaningful automated coverage must exercise schedule/window boundaries, the
300-second outcome and precedence, cap/reservations, skipped bursts, paused
warnings/hazards, queued/exhausted choices, and full restart of the combined run.
Retain regression checks for fixed enemy stats/behavior and normal module/terrain
interactions; do not merely assert a copy of the authored schedule.

Run cargo fmt --check, cargo test --locked, and
cargo clippy --all-targets --locked -- -D warnings. Use cargo dev for native
gameplay checks, deliberate death/restart, and the complete final encounter.

Measure the highest playable density on the target Mac with actual pursuit,
terrain navigation, shots, and feedback. Record hardware, OS, commit/profile,
resolution, sampling duration, actual enemy counts, median/p95/p99 frame times,
and hitches above 33.3 ms, excluding warmup and paused/terminal frames. Use the
existing provisional 60 FPS target (p95 at or below 16.7 ms). Any increased cap
must be measured before acceptance. Harness invulnerability or population
replacement must be disclosed and excluded from gameplay-success evidence.

Record final tuning, attempts, observations, validation results, remaining limits,
and the explicit gate decision in docs/playtests.md. Review and commit the
implementation separately. The original five-session allowance is provisional;
reassess after the first combined playtest rather than expanding feature scope.


## Approved minimal amendment after manual feedback

The user chose opening pacing and honest exhausted-catalog UI for PR #11, with
handling/arena/progression redesign deferred. This supersedes the initial
0–45-second opening and 45–105-second first pressure phase above: opening is
0–30 seconds with warnings at 3/13/23, then 3 every 8 seconds from 30 until the final
six-second lull at 99. Later phases retain their original timing and sizes.
Total 46 bursts request 262 enemies before rejection or skipped outcomes.

Immediately after no eligible upgrades remain, show Build complete and acquired
upgrades without level/XP progress. Retain internal XP accounting, all existing
upgrade rules, and the six-upgrade catalog. A four-slot build limit was proposed
but not accepted for this PR. See the final schedule, evidence, and follow-up
Linear links in docs/playtests.md. Another human playthrough remains required.
