# DRO-37: Collision bombs and mothership spawning

Implement the next dependency-ready child of DRO-22 using its confirmed Repulsor
counterplay and catalog-isolation requirements. Numeric values are provisional.

## Bombs

A 20-hull bomb carrier follows ordinary flight and swept contact. Contact consumes
it without XP/loot/kill credit and attaches one bomb if none is present. Attachment
has no immediate damage and ignores shields/invulnerability; subsequent carriers
cannot stack or refresh the bomb. The visible fuse lasts three active seconds
from attachment, including on long frames. Detonation consumes the bomb exactly
once and attempts 25 damage through ordinary shield/invulnerability rules.
A shield block or invulnerability consumes the explosion without delayed retry.
Destroying a carrier before contact prevents attachment and gives one ordinary
kill reward. Restart, return, mission cleanup and terminal outcomes clear bombs.
Upgrade choices pause the fuse.

Repulsor is selectable only in the catalog loadout. It uses standard slot toggles,
activation threshold, jam lock and battery depletion, drains 8 energy/second and
pulses immediately when ready while powered, then every 2 active seconds. It
removes an attached bomb without an explosion or reward. Cooldown runs only during
gameplay, cannot be reset by toggling, and resets with the round. A ready powered
pulse wins over detonation on the same frame. This is the bomb-removal foundation;
pushback, Repair and the complete module catalog belong to DRO-39. Repulsor is
excluded from campaign purchases, saved loadouts and upgrade offers.

## Motherships

A 100-hull mothership uses ordinary flight at a 120-unit speed cap, approaching
until within 350 units. Its collision hull remains the existing enemy hull;
a distinct faceted body with launch ring identifies it. Every six active seconds
it attempts one child, with a full 1.2-second spawn warning. It cycles Chaser,
Fast, Rammer, Slower, Jammer, Bomber, never Mothership. Admission and activation
both require arena, terrain/hazard, player and occupied-hull clearance. Candidate
sites form a bounded ring around the parent. Pending warnings count toward the
shared WaveConfig cap (30 by default); no catch-up burst on a hitch. A warning
owns a parent reference; parent death cancels pending launches even on the delivery
frame. Launched children remain alive after parent death. Pauses freeze cadence;
restart/return/terminal clear pending launches through normal transient cleanup.
Spawn counters account for requested/admitted/rejected/activated/cancelled children.

## Architecture and presentation

Reuse shared waves::safe/spawn_half validation and SpawnWarning delivery; extend
warnings with an optional separate parent component rather than changing every
warning constructor. bombs.rs owns bomb/pulse state and damage resolution;
mothership.rs owns parent cadence and warning admission. Existing ordered combat
systems resolve lethal hits before contact/spawning. Cached silhouettes and a
compact HUD expose fuse, dislodge/detonation, pulse cooldown, spawn kind/countdown.
Add Collision bomb, Mothership, Mixed ordnance scenarios after existing scenarios.
Campaign wave tables, shop catalog, upgrade offers and save schema remain unchanged.

## Validation

Production-system tests at 30/60/144 Hz cover attachment, nonstacking, damage,
shield/invulnerability, powered/empty/jammed pulse, pause/reset/terminal, typed
kill credit, cadence, caps, parent death, clearance and nonrecursive children.
Run complete locked tests, format and strict Clippy. Native catalog fixtures at
1120x720 and 640x480 exercise selector, countdown, pulse, detonation, spawning,
restart and mixed roster with explicit synthetic positioning. Inspect screenshots
and record evidence/limits in docs/playtests.md. Human balance acceptance remains
pending and does not authorize campaign introduction.
