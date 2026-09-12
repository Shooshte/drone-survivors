# DRO-30: flight room and following camera

The user selected DRO-30 first with DRO-31 refinement in parallel, then approved
“Larger arena + following camera”: start at 1920×1080, retain the current zoom,
add off-screen charger/warning indicators, and compact the HUD.

## Layout and flight

Double floor dimensions to 1920×1080; retain the 300-unit ceiling and scout start.
This offers roughly 2.6 seconds across the short axis at 420 units/s before body
clearance, compared with 1.3 seconds today. Keep the established DRO-29 flight
model and every enemy/weapon stat. Move chargers to x = ±560. Keep charger size,
reserves and recovery unchanged. Retain a full-height divider at x = 120, a
140-unit central electrical opening, and a wider safe detour around its south end.
Extend the north divider to the new boundary and the south divider to z = 300;
the detour runs around z = 410. Move low cover toward the outer corners so the
interior offers sustained travel and room for banked curves.

Authored layout coordinates must drive route dots, enemy navigation corners,
charger setup and validation pilots consistently. Preserve real swept body
collision, safe warning activation and empty-battery access to both chargers.
Keep warning timing, perimeter spawn rules and wave schedule initially; measure
pressure before considering a separate tuning change. No damage reduction is
justified yet. Progression rules belong to DRO-31 and are unchanged here.

## Camera and navigation

Use an orthographic camera with the current fixed orientation and viewing scale
(1120 minimum width, 800 minimum height). Translate with the scout in world X/Z;
retain a fixed altitude reference so thrust does not move the ground under the
camera. Follow promptly without heading rotation or speed-dependent zoom, freeze
through choices/outcomes, and snap back with restart. The camera need not show
the full arena; preserving scout and warning size is the purpose of following.

Screen-edge indicators identify off-screen LEFT/RIGHT chargers and incoming
warnings, using text/shape as well as color. Derive positions from the actual
camera projection, handle resize and off-screen depth, and hide them during
upgrade overlays. Avoid indicator overlap with the top/bottom HUD; aggregate
warnings by direction if needed for legibility. Do not label ordinary enemies
as incoming warnings. Keep the ground marker and vertical guide.

## HUD

Condense the current tall overlapping status panels into a shallow header and
footer at 1120×720 and 640×480. Show hull, timer, hostiles, kills, energy/net flow,
charger reserves and recovery, all four module states, shield readiness,
progression and hazard status. Retain essential controls and failed-activation
feedback. Full instructional copy can be shortened without removing meaning.
Keep central flight space free of status text; upgrade choice and outcome screens
remain readable. No new visual assets or dependencies are required.

## Verification and acceptance

Add behavioral tests for long clear flight lanes and banked curves at baseline,
mobility and heavy builds, measured across 30/60/120 Hz. Verify both shortcut and
detour directions on empty energy, charger depletion/recovery/restart, safe enemy
navigation/spawns, and camera projection/indicators across resize and reset.
Compare ordinary encounter pressure using deterministic pilots without granting
health, damage or XP. Record route times, curve extents, collisions/contact causes,
nearest-threat distance and outcomes; separate fixtures from human observations.
Run formatting, all tests and Clippy, native route and charger checks, and native
captures at both supported sizes. A human must still judge whether sustained fast
travel is enjoyable and whether the expanded encounter feels too sparse; report
that gate honestly rather than treating automated survival as acceptance.
