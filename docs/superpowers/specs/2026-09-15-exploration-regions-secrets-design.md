# DRO-20: Exploration, regions, and secrets

Status: concrete proposal for user review; implementation has not started.

## Confirmed decisions

- All acts retain the current placeholder arena layout. Preserve terrain,
  passages, arena bounds, objective placements, and charging-node locations.
- Secret discoveries unlock their campaign rewards immediately, including on
  attempts that later fail or restart. Save those discoveries immediately.
- A secret exposes a purchasable hub upgrade. Its purchased bonus survives
  successful missions but is lost on defeat or manual mission restart.
  The blueprint remains unlocked, allowing another purchase.
- Restart counts as defeat for forfeiting this bonus. Existing permanent
  passive ranks and module ownership retain their existing behavior.

## Proposed content

Assign one resource profile to each act, using the same arena and resource-site
positions. All numbers are provisional playtest values.

| Act / region | Chaser salvage drop | Each of three component caches | Each of six charger reserves |
| --- | --- | --- | --- |
| 1 / Scrapyard | 50% chance of 1 salvage | 1 component | 200 energy |
| 2 / Ruins | 25% chance of 1 salvage | 2 components | 200 energy |
| 3 / Power station | 25% chance of 1 salvage | 1 component | 300 energy |

Retain current recharge speed, depletion recovery rules, reward settlement,
mission objectives, and encounter waves. Generate briefing resource summaries
from the same profile data used at launch.

Turn the three existing component caches into optional discoveries. Each awards
30 run XP once per attempt, alongside its existing resource collection.
Revisiting a collected cache cannot pay again; launch/restart restores caches.
The existing standalone 30-XP pickup remains available.

- Northwest cache: recover the **Reserve battery** blueprint, once per campaign.
  Later discoveries of this blueprint still award that attempt's XP/resources.
- Southeast cache: discover a route to the current act's finale (04, 08, or 12),
  permanently granting access without first completing both branch missions.
- Central cache: XP and region resources only.

The normal campaign unlock path remains available. Secret access does not mark
any mission complete. All 12 distinct victories are still required to complete
the campaign, and previously accessible missions remain replayable.

## Purchasable secret upgrade

**Reserve battery:** costs 20 salvage and 1 component; adds 25 maximum battery
capacity after existing permanent passive bonuses and before temporary run
modifiers. One purchase can be active at a time; no stacking or additional
ranks. Buying it debits the wallet exactly once and saves immediately.

Show the blueprint and purchase state in the hub's upgrade interface, with
explicit wording that the bonus is lost on defeat or restart. Success retains
the purchase; defeat clears it before saving results; restart clears it before
calculating the next attempt's stats and saves the loss immediately. A normal
application close does not itself count as defeat. Existing ordinary upgrades
remain permanent.

## Discovery interaction and feedback

Use automatic proximity collection with terrain line of sight, including swept
movement checks so a fast crossing cannot miss a cache. Retain the existing
50-unit 3D collection radius and reachable cache positions. Resource caches
remain visible world objects; no new interaction key or mandatory exploration
objective. Show a concise discovery/reward notification and explain the secret
unlock in the hub/mission selection after discovery.

Pause discovery during menus, choices, and results. Restart and fatal damage
take precedence over new collection on the same frame. Already granted
discoveries survive subsequent failure, restart, and application reload.

## Implementation boundaries

Represent reusable arena content pieces and region resource profiles as data,
preserving the assembled placeholder layout. Use shared descriptors for sites,
charger placement, runtime configuration, and briefing summaries. Keep region
and discovery rules separate from rendering and the mission lifecycle.

Persist blueprint ownership, active purchase, and discovered finale routes.
Load existing version-1 saves with those fields absent as an undiscovered
campaign. Validate secret-enabled mission progress against normal OR discovered
access, including selection and saved attempt history. Never invent completion
for bypassed prerequisites. Use the existing atomic save and write-error flow;
an immediate save failure must be visible and retryable.

## Verification and delivery

- Test one discovery per attempt, campaign reward idempotence, pause/terminal/
  restart precedence, fast crossings, terrain blocking, and reachable sites.
- Test actual region drops/cache amounts/charger reserves against briefing data.
- Test purchase cost, insufficient funds, duplicate purchase, success retention,
  defeat/restart loss, repurchase, and composition with existing upgrades.
- Test immediate discovery persistence, immediate restart-loss persistence,
  legacy saves, secret route reloads, and normal progression without secrets.
- Run `cargo fmt --check`, `cargo test`, and `cargo clippy --all-targets`.
- Use `cargo dev` for native gameplay and presentation checks at 1120x720 and
  640x480. Record fixture overrides and any remaining human playtest limits.
- Record results in `docs/playtests.md`, review, make coherent commits, push
  `codex/dro-20-exploration`, and open a PR against `main`.

Update Linear with the agreed scope before implementation.
