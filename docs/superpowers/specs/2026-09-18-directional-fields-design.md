# DRO-38: directional fields and hull-repair sites

Implement in the isolated catalog arena only. Campaign layouts, progression,
saves, shops and spawns are unchanged. User-approved field behavior: permanent
fields stay present, cycling fields turn on/off, and all movement effects stop
on exit. User confirmed one repair charge per site per round, restoring up to
35 hull, with no charge consumed at full hull; restart restores the charge.

## Chosen approach

Add one Environment practice selector entry with no enemies and one Environment
pressure entry with the existing fast pursuer and slowing beam. Both expose the
same permanent eastward field, cycling eastward field and repair site. Keep the
existing twelve scenarios unchanged. A resource installed only by catalog owns
fixtures, active time and repair availability; reset activates it only for these
two scenarios. No new save fields or dependencies.

Fields scale east/west ground travel, not stored flight velocity: east +40%, west
-40%, perpendicular/vertical unchanged. Sample membership per bounded 120 Hz
physics substep and apply before swept terrain collision, preserving the actual
player path for other systems. Leaving restores ordinary travel on the next
physics substep (at most 1/120 s), with no lingering velocity/stat modifier.
Overlap uses one bounded modifier, never a product. The field covers full flight
height; floor outlines and arrows mark its horizontal bounds. Temporary fields
run six seconds on/four off in active gameplay time and pause during choices.
Flight configuration, module boosts and beam slow compose through ordinary
flight before the field's ground-travel modifier. Enemies are unaffected.

A repair site at height 90 triggers within 70 world units and clear line of sight,
including swept crossings. It restores min(35, missing hull) once, cannot revive
a dead player, and waits while at full hull. A player already inside when damage
occurs may consume the still-ready charge. Site consumption follows combat, so
lethal combat always takes precedence. Restart/return/launch restores fixtures.

## Presentation and validation

Show field outlines, east arrows and an on/off cue, plus a repair cross/ring that
changes to a depleted cue. A compact environment status panel states direction,
permanent/cycle state and countdown, inside/outside status, site readiness and
distance. Selector copy gives the numerical rules and repair height/range.

Use behavioral unit/integration tests for axis response, overlap bounds,
entry/exit, collision, frame rates, cycle/pause/reset, repair cap/full hull/death,
swept proximity and line of sight, and absence from other scenarios/campaign.
Run all tests, fmt and strict Clippy. An opt-in native fixture uses synthetic
positions and damage to check real transitions and viewport bounds at 1120x720
and 640x480, capturing readable states. Clearly label agent evidence; human
acceptance and campaign introduction remain gated.
