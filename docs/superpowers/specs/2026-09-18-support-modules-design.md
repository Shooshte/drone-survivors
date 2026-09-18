# DRO-39: Repair and Repulsor

## Scope and authorization

DRO-39 is the next ready child of DRO-22. Its confirmed September 15 scope approves Repair and Repulsor, including powered pulses that dislodge bombs. This delivery completes those designs in the isolated catalog arena. Campaign introduction and human acceptance remain gated; numeric tuning is provisional.

## Behavior

Repair restores 6 hull per powered second, costs 12 energy/s and cannot exceed the effective maximum hull. Preserve fractional paid repair between frames, but discard credit at full hull so it cannot be banked. Only actual supplied time counts, including the paid portion of a depletion frame. Repair cannot revive terminal players or bypass a jammer. Full hull still drains power while enabled; the player can turn it off.

Repulsor retains the existing 8 energy/s and 2-second pulse interval. Each powered, unlocked pulse removes an attached bomb and pushes living enemies within 180 units in 3D, with solid terrain blocking the pulse. Use an outward velocity impulse (260 units/s) through normal collision-aware flight, without damage, teleporting, kills or rewards. Handle coincident centers without nonfinite vectors. The existing active-game cooldown continues while switched off and cannot be reset by toggling; upgrade choices freeze it. Depletion prevents a free pulse; only one pulse is emitted per update, with no hitch catch-up burst. Existing bomb deadline precedence stays intact.

Both modules use the four ordinary slots and the existing activation threshold (10 energy, not an extra debit). Enemy locks turn modules off until the player re-enables after recovery. Restart, return and launch clear repair credit and pulse state. Existing modules and four run-upgrade opportunities remain unchanged.

## Implementation boundaries

Keep campaign ModuleKind::ALL at four types; catalog choices contain all six. Both added types are excluded from the serde save codec and rejected by campaign purchases/loadouts. Add tuning to ModuleConfig so later catalog upgrades can alter a stable baseline. Expose actual paid repair time from energy preparation, and apply repair after damage/terminal resolution but before power commit. Extend the existing bomb pulse path to share one cooldown and one visual cue with enemy repulsion.

Use concise benefit, drain, activation and operational-limit copy in the selector; retain readable four-slot HUD and pulse ring. Avoid adding a separate permanent healing system or changing enemy pursuit AI. Alternatives considered: direct positional knockback risks terrain tunneling; a separate pulse timer duplicates bomb behavior. The existing power pipeline and flight velocity provide smaller, consistent boundaries.

## Evidence

Test paid partial frames, multiple frame rates, full-hull cap, no banking, empty power, disable/jam, pause, reset/return, terminal frames, pulse range/line-of-sight, collision safety, bomb removal and campaign/save exclusion. Run formatting, all tests, strict Clippy and native fixtures at 1120x720 and 640x480. Record agent fixtures and synthetic inputs explicitly; do not claim human balance acceptance.
