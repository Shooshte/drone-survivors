# DRO-29: coordinated scout turns and response

The user approved option 1: Q/E banks the scout and gradually turns its nose into
the curve. A/D retains direct yaw. This implements the high-priority handling
follow-up before DRO-30 establishes final arena dimensions.

## Rules and boundaries

- Bank assistance follows actual signed bank, reaching at most 90 degrees/second
  at full bank. Q turns left, E right. A/D takes priority over assistance while
  held, allowing deliberate counter-steering. Releasing bank levels the scout
  and fades assistance with the visible attitude. Opposite Q/E cancel input.
- Heading changes rotate thrust and the model, never existing velocity. Keep
  bounded 120 Hz physics steps, swept terrain collision, and horizontal speed cap.
- Increase scout horizontal rotor acceleration to 2.5 times the previous value;
  retain the 420 speed cap, gravity, vertical thrust and passive drag. Increase
  attitude response by 2 times only when commanded tilt opposes existing tilt.
  Normal tilt, leveling and direct yaw rates stay at 240/300/240 degrees/second.
- Mobility multiplies the new horizontal thrust and cap. Armor/rounds retain
  their translational penalties; Agile frame retains its angular improvement.
  Bank assistance follows the angular improvement too. Basic handling is free.
- Enemy flight must use its exact previous unassisted tuning. No enemy, weapon,
  map, camera, progression, or charger balance changes.

## Validation targets

Use an unobstructed physics fixture to separate wall contact from control response.
Measure at 30/60/120 Hz: banked forward curves in both directions, immediate
counter-yaw, bank reversal, level/coast, and opposite pitch braking from 300 units/s.
Target visible tilt reversal within 0.15 seconds, reduced forward velocity within
0.1 seconds, baseline stop within 1 second, and heavy armor plus rounds stop
within 1.5 seconds. Acceleration should reach 140 units/s by 0.5 seconds.
Measurements must include mobility and heavy builds and report stop distances.
Keep existing pause/reset, bounds, terrain, empty-energy and enemy regressions.

Native checks exercise rendering and normal input. Human judgments about enjoyable
evasive curves and intentional stopping remain a documented playtest gate; an
automated pilot cannot supply those observations. Any native fixture must be
explicitly selected and must not modify ordinary play.
