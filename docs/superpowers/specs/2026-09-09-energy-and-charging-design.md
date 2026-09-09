# DRO-8 — Energy and charging

Source: https://linear.app/drone-survivors/issue/DRO-8/04-energy-and-charging

Approved in conversation on September 9, 2026, with authorization to implement
on a separate branch, commit at milestones, push, and open a PR against main.

## Gameplay

Preserve the three-minute survival encounter, flight physics and controls, basic
weapon damage/targeting/range, and all existing death/restart behavior. Add a
100-capacity battery, initially full with weapon overdrive off. Key **1** toggles
overdrive: twice the normal automatic firing rate, draining 10 energy/second
while enabled even without a target. Enabling requires at least 10 energy;
disabling is always allowed. Depletion clamps energy to zero and switches
overdrive off. All ordinary firing and flight, including thrust controls,
remain usable. Recovery never automatically enables overdrive.

Two identical persistent charging nodes occupy opposite sides of the arena at
(-280, 0, 0) and (280, 0, 0). Each has a visible cylinder of radius 90 spanning
world heights 0 through 160. The drone's center must be inside the cylinder;
its radial and height boundaries are inclusive. Nodes are non-solid, unlimited,
and provide no healing or protection. Enemies continue pursuing and damaging
through fields. Camping is a valid tactic to evaluate in playtesting, not
prevented with cooldowns or forced node switching.

Charge automatically at 25 energy/second while inside either field. Overdrive
continues consuming while charging, producing a net gain of 15 energy/second.
Overlapping fields do not multiply charging. There is no outside regeneration.
Clamp the combined rate once to [0, capacity], avoiding artificial depletion
from applying drain before recharge. Ten seconds empties a full active battery;
four seconds fully charges an empty inactive battery. Values are provisional.

## Scheduling and lifecycle

Use the existing ordered Bevy gameplay schedule. Reset wins over toggle,
movement, energy, and combat on its frame. Evaluate charging after movement.
Resolve contact damage and survival before energy updates and firing so a
terminal frame cannot recharge, drain, toggle, or fire after the outcome.
Accept activation against the stored energy before that frame's recharge.
Apply toggle, then the combined energy rate, then choose the firing cadence.
Membership samples the center after each movement update; crossing a field
between samples does not bank charge. Key holds do not repeatedly toggle.

Retiming an active weapon cooldown preserves its fractional progress when
switching cadence. Switching never resets the cooldown or grants bonus shots;
normal hitch and target-loss protections remain. Already emitted projectiles
keep their damage and motion. At zero, ordinary cadence resumes immediately.
Death/survival freezes battery, overdrive, and transient activation feedback.
R restores full energy, off state, and clears charging/rejection feedback;
node entities and reusable visual assets persist without duplication.

## Presentation

Show an energy bar and numeric current/capacity, overdrive ON/OFF/EMPTY,
charging state/net rate, and the 1 key hint. An activation rejected below 10
energy displays a short readable explanation. Visible floor/top boundaries
and translucent sides communicate each field's height without obscuring the
drone or enemies. Highlight the occupied node. Preserve combat HUD readability
at the supported minimum 640 × 480 window size.

## Architecture and validation

EnergyPlugin owns battery, tuning, node components, reset and energy updates;
EnergyScenePlugin owns reusable field visuals and HUD. Combat consumes only
the effective firing-rate multiplier. Do not build loadouts or general module
machinery ahead of DRO-9. No new dependencies are needed.

Headless ECS tests cover rate/frame-size consistency, simultaneous charging,
capacity/zero clamps, inclusive geometry and just-outside points, both nodes,
non-stacking overlap, held and rejected input, manual recovery, flight and
basic firing at zero, cooldown transitions, terminal ordering and repeated
restart. Verify presentation state and asset reuse where useful. Run cargo
fmt --check, cargo test --locked, cargo clippy --all-targets --locked -- -D
warnings, and native cargo dev checks. Record actual evidence and limitations
in docs/playtests.md. Review and commit milestones; push and open the PR.

Four-slot loadouts, other powered abilities, node cooldowns, XP/upgrades,
obstacles, and campaign changes are outside this ticket.
