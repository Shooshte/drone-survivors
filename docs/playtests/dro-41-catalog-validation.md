# DRO-41 — Bounded catalog combinations

September 18, 2026. Agent validation of the isolated arena, **not human
acceptance**. The campaign gate from DRO-21 remains deferred. Campaign saves,
spawns, shops and upgrade offers are unchanged.

## Reproduce a combined round

```sh
cargo dev --locked -- --validate catalog
```

From initial Flight practice, press Left once for **Combined catalog** (15/15).
Choose modules with 1–4; U opens upgrade previews. Enter starts the round.
The three waves at 1/16/31 seconds each request all seven enemy kinds. Spawn
safety and the shared 30-enemy cap still apply, including mothership launches.
Both directional fields and the single +35 hull repair site are enabled. The
default 60-second duration includes no time spent in upgrade choices.

The new scenario leaves each focused drill intact. No numeric balance changes
are bundled merely to make the validation pass. Results below distinguish
observations from evidence sufficient to change tuning.

## Native presentation and lifecycle

```sh
DRONE_COMBINATION_SMOKE=1 DRONE_CAPTURE_DIR=/tmp/dro41-native-large cargo dev --locked -- --validate catalog
DRONE_COMBINATION_SMOKE=1 DRONE_CAPTURE_MINIMUM=1 DRONE_CAPTURE_DIR=/tmp/dro41-native-small cargo dev --locked -- --validate catalog
```

Both fixtures passed. Synthetic keyboard input selects Overdrive, Shield, Repair
and Repulsor, then Efficient coils, Heavy rounds, Rapid repair and Wide repulsor.
The normal preview XP opens the four ordinary one-card choices before encounter
time advances. The pilot follows a west-side field loop, powers offense/shield/
Repulsor and requests Repair below 85 hull. It supplies no health, energy,
enemies, kills or wins. It samples 3/10/20 active seconds and then tests R/Tab.

All seven kinds were observed at both window sizes. At 20 seconds the compact
run had 65 hull and six kills; the large run had 90 hull and seven kills. Native
variable frame timing and waypoint timing make these presentation samples,
not controlled performance comparisons. Both depleted the initial battery;
free basic fire and flight continued. Restart restored hull, disabled modules,
repair availability and preview budget; return removed active environment/effects.

Visible text bounds passed at setup, preview, all four choices, three combat
samples and return. Captures were inspected for wrapping, silhouettes, beam/bomb
cues and combined HUD readability. The compact capture simultaneously shows an
active beam, attached bomb, field/repair guidance, empty battery and four modules.
The existing environment fixture also passed at 640×480 after updating its
wraparound through the added scenario. Each successful process emitted the
existing Bevy winit shutdown warning; no fixture assertion failed.

- [1120×720 log](evidence/dro-41/native-1120x720.log)
- [640×480 log](evidence/dro-41/native-640x480.log)
- [Environment regression](evidence/dro-41/environment-640x480.log)

![Combined selector at 640×480](../images/dro-41-setup-640x480.png)
![Four-card comparison at 640×480](../images/dro-41-preview-640x480.png)
![Beam, bomb, environment and empty modules at 640×480](../images/dro-41-combat-640x480.png)
![Combined combat at 1120×720](../images/dro-41-combat-1120x720.png)

## Demonstrated enemy responses

These are production-system agent checks. Controlled fixture setup is used for
individual attack timing; they do not establish that a human will notice or
successfully execute the response under mixed pressure.

| Enemy | Response and supporting check |
| --- | --- |
| Chaser | Shoot while moving; the combined round uses ordinary auto-fire, contacts and navigation. Empty-loadout controls retain viable free flight/fire. |
| Fast pursuer | Change heading/altitude and use solid cover rather than matching a straight chase; variant flight regressions verify faster pursuit and solid geometry. Mixed rounds exercise it with ordinary shots, Mobility and Repulsor. |
| Double-impact rammer | Leave the warned line or use cover. `missed_charge_retreats_without_spending_budget_and_terminal_states_freeze_flight` verifies a committed miss retreats; `rammer_shield_blocks_spend_hits_but_invulnerability_does_not` verifies Shield as an alternative. Two actual impacts remain separately warned at 30/60/144 Hz. |
| Slowing beam | Break sight/range during windup or kill the source. `control_keyboard_escape_breaks_attacks_in_both_isolated_scenarios` compares stationary and keyboard escape at 30/60/144 Hz; escape avoids the slow. Cover and lethal-source tests also pass. |
| Module jammer | Same escape/source response; after a hit, re-enable the announced slot after its lock expires. The keyboard escape comparison avoids locks, and module recovery tests verify OFF until a fresh toggle. |
| Collision bomber | Kill before contact, or reserve powered Repulsor/Shield for attachment. `bomb_carrier_destroyed_before_contact_awards_one_typed_kill_and_never_attaches`, `bomb_powered_repulsor_removes_bomb_and_wins_deadline_tie` and shield/block tests exercise each response. Jam, brownout and cooldown prevent free removal. |
| Mothership | Destroy the parent to stop its launch stream. `parent_death_cancels_warning_before_its_delivery_deadline` and `ordnance_lethal_projectile_cancels_parent_delivery_at_every_frame_rate` exercise cancellation, including lethal-frame ordering; delivered children remain threats. |

The counterplay regression suite also covers source death, overlapping locks,
shield/invulnerability, terrain, finite power and terminal-outcome ordering.
Human checks should test whether these responses stay legible during all-seven
pressure, particularly bomb removal while Repulsor is jammed or depleted.
