# DRO-35: Fast pursuer and double-impact rammer

## Scope

Build on merged DRO-34, in a separate worktree and PR. The user approved two
separate impacts with retreat and warning; arena-only content expansion remains
authorized while campaign introduction stays gated. No further split is needed.

## Behavior and provisional tuning

- Fast pursuer: 350u/s horizontal cap, 20 hull, existing rotor-flight pilot and
  terrain navigation. Ordinary chaser remains 260u/s. Cyan body with paired fins
  distinguishes it from the orange chaser; turn, change altitude and use cover.
- Rammer: 80 hull; approach at 260u/s. Within 280u, brake and show a full 1s
  warning, then charge toward a locked point beyond the target at a 420u/s cap.
  It cannot steer the charge toward the player's new position. After contact or
  a 1.6s missed charge, retreat using the same physical flight/navigation.
- No contact damage in approach, warning or retreat. Rearm only after 1.2s of
  retreat and at least 180u separation, then warn again. Each accepted hull hit
  or shield block spends one of two impacts. Shared invulnerability rejects an
  impact but still ends that charge. Second accepted impact destroys the rammer
  without granting a kill, XP or loot. Shooting/hazard kills follow existing
  combat outcome/drop rules exactly once. It can be killed during any phase.
- A magenta armored body, state-colored ring and two visible impact indicators
  expose type, remaining budget and warning/charge/retreat state. Reuse cached
  meshes/materials and preserve the right material after hit flashes.
- Add Fast pursuer, Double-impact rammer and Mixed pursuers selector scenarios.
  Store the selected enemy kind with each spawn warning, preserving ordinary
  spawn clearance/caps/activation checks. Existing scenarios, campaign waves,
  economy balances and save formats stay unchanged.

## Implementation choices

Use an optional Rammer component for the small state machine and per-type tuning
inside CombatConfig. Feed desired targets and speed profiles into the existing
rotor integrator. Replacing that integrator or teleporting the retreat would
break terrain/handling rules, so neither is needed. Use production wave spawning
with an optional arena-only roster rather than a second spawning implementation.

New-type contact sweeps pair player and enemy motion segments over their shared
time intervals; use their actual collision envelopes and reject contact through
occluding solids. Ordinary chaser contact behavior remains unchanged. Rammer
state transitions discard hitch overshoot so a warning cannot be skipped in an
unseen frame. Terminal states and upgrade choices pause the state machine.

## Verification

Test catalog opt-in and campaign isolation; real warning activation; faster
pursuit without terrain tunneling; two separated impacts; hit/block/invulnerability
budgets; missed charges; warning/retreat immunity; 30/60/144 FPS and hitches;
reset and paused/terminal states; player-caused kill/drop events exactly once.
Run all Cargo checks and native arena screenshots/fixtures at both supported sizes.
Document synthetic setup and limits. Human acceptance remains pending.
