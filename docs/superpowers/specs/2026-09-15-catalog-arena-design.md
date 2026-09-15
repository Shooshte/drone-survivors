# DRO-34: Isolated catalog arena

## Scope and authorization

The user authorized DRO-22 implementation in a test arena while keeping campaign
introduction gated on the deferred DRO-21 human playtests. DRO-22 is now an
umbrella for DRO-34 through DRO-41, each intended for its own PR. This first PR
delivers the arena foundation using the existing catalog only.

## Design

Use `cargo dev -- --validate catalog` to open an interactive selector. Reuse the
existing world, production flight/combat/energy/upgrades and their reset boundary.
Do not install campaign, economy settlement or save plugins. No owned inventory,
campaign rewards or persistent changes exist in this mode.

Three baseline scenarios make the first PR independently usable: Flight practice
(no enemies), Single pursuer (one warning/spawn), and Small swarm (three waves of
five). Existing terrain, timed hazard, chargers and exploration XP remain present.
Each round lasts 60 active seconds by default; `--seconds` changes that limit.
Ordinary hull, battery, damage, naturally earned upgrades and terminal states apply.

The selector lists the scenario and response to practice, plus four loadout slots.
Left/Right or scenario buttons select a scenario. Keys 1-4 or slot buttons cycle
that slot through empty and the existing modules, skipping modules equipped in
other slots. Enter or Launch starts a fresh round. During a round, module hotkeys
keep their ordinary meaning; R restarts, and Tab returns to the selector, including
from upgrade choices and terminal states. Tab avoids the upgrade Skip binding.
Escape retains its existing application-quit behavior.

Launch, restart and return use `MissionBoundary` to restore the production
baseline, hull, energy, chargers, drone position/momentum, upgrades and hazard
state, and remove combat transients. Selection pauses gameplay. Selection changes
cannot leak held launch/slot input into combat. Keep the scenario, loadout and
duration on restart. The arena resource and selector are installed only in catalog
mode; normal campaign and prior validation entry points remain unchanged.

Keep controls in a compact overlay using existing Bevy UI styling. Verify selector,
combat help and upgrade/terminal transitions at 640x480 and 1120x720. New catalog
children can extend scenarios and choices without exposing them in the campaign.

## Alternatives considered

- Reuse a campaign mission: simpler navigation, but introduces rewards, save and
  unlock interactions that are inappropriate for content experiments.
- Only add command-line scenario flags: small implementation, but fails the
  selectable arena requirement and makes iterative comparison cumbersome.
- Chosen: an isolated interactive selector over production gameplay.

## Verification

Regression tests cover opt-in routing, paused selection, keyboard and mouse
actions, unique slot editing, production scenario spawns, full restart from a
modified/paused run, return/relaunch and the absence of campaign/save resources.
Run the full Cargo suite, formatting and strict Clippy, then native checks at both
window sizes. Record synthetic native input explicitly; these checks do not
satisfy the human playtest gate.
