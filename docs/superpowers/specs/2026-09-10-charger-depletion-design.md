# DRO-32: Temporary charger depletion

Approved by the user on September 10, 2026. Implementation starts from main
`2012fc3` in the `codex/dro-32-charger-depletion` worktree.

## Purpose

Make charging a useful stop that eventually requires relocation or power
conservation. Both stationary overdrive and overdrive/shield currently survive
the full encounter at either charger. Enemy behavior, waves, XP, scout handling,
arena geometry, module costs, and campaign systems stay outside this change.

## Approved rules and initial tuning

Each existing charging field starts with an independent 200-energy reserve.
Delivery remains capped at the existing 25 energy/second. Deduct only energy
actually supplied to the battery or active modules. A full battery with overdrive
running consumes 10 reserve/second; a full battery with all modules off consumes
none. The drone's ordinary battery capacity and activation threshold remain
100 and 10, subject to existing upgrades.

A field at zero reserve supplies no energy. Being inside a field prevents its
recovery even if the battery is full, modules are off, or an overlapping field is
selected as the source. After eight uninterrupted active-gameplay seconds outside
that field's existing 3D charging volume, it recovers 10 reserve/second, capped at
200. Time before the delay expires does not refill reserve. Re-entry resets the
absence timer without restoring or deleting remaining reserve. Partial reserves
are usable immediately. A brief exit and return cannot refill a depleted field.

An empty field takes 28 continuous seconds away to recover fully. The proposed
reserve provides eight seconds of maximum delivery, or 20 seconds sustaining
overdrive at a full battery. Including the initial battery, an uninterrupted
overdrive camp has about 30 powered seconds; overdrive plus shield has about
16.7 seconds before upgrades. These are energy budgets, not survival predictions.
Refine numerical values from route and encounter probes while retaining these
rules and recording the chosen values.

Overlapping fields share the same 25/s delivery ceiling. Select a deterministic
nonempty source and debit only that source. Exhausting it can transfer delivery
to another containing nonempty field without granting extra delivery time or
double charging. All containing fields remain occupied for recovery purposes.

Module activation still uses stored energy before this update's charging. When
battery energy runs out, modules switch off and never restart automatically.
Handle battery-full, battery-empty, reserve-empty and recovery-delay boundaries
chronologically so long updates conserve energy and agree with subdivided time.
Shield recharge receives only the time during which its module was powered.

## Presentation

Keep both charger statuses visible before arrival. Identify them consistently
as LEFT and RIGHT in the HUD and add a reserve indicator to each world field.
Show remaining reserve and explicit status: READY, IN USE, DEPLETED / LEAVE TO
RECOVER, recovery delay countdown, or RECOVERING. The battery display must not
claim positive charging from an empty field. Use text and changing fill/shape
alongside color; avoid relying on color alone. Verify that the two compact status
rows fit with the existing hull, energy, modules and XP at 640 by 480.

## State and integration

Keep reserve and absence state with charger entities and tuning in one focused
configuration. Keep chronological energy calculation separate from presentation.
Extend the existing power staging: prepare candidate battery, module and charger
state before damage, then commit only if the frame remains Playing after fatal
damage and survival resolution. Fatal damage retains precedence over survival.

Choosing, Dead and Survived freeze reserve and recovery clocks. R restores both
full reserves and clears absence timers as part of the ordinary complete restart.
No reserve state survives a new run. Existing world geometry and energy occupancy
semantics remain in force; routes require no powered mobility.

## Validation and completion evidence

- Unit/ECS checks for actual delivery with a full or empty battery, toggles,
  all-module drain, depletion, independent reserves, partial recovery, the exact
  absence threshold, brief boundary crossings and overlapping fields.
- Compare 30/60/120 Hz and larger updates around battery and reserve boundaries;
  assert conservation, bounded values and correct powered shield time.
- Extend pause, restart and fatal/survival-frame regressions to charger state.
- Preserve and rerun the current camping baseline, then compare both chargers
  with overdrive and overdrive/shield using normally earned offensive upgrades.
  Record actual reserve delivery, powered time, deaths/survival and choices.
- Compare a moving keyboard pilot that visits both fields and conserves power;
  use actual flight, normal damage and normally earned XP. Record route travel,
  charging stops and outcomes; identify scripted evidence as diagnostic only.
- Native screenshots at normal and minimum size, plus runtime smoke checks for
  depletion, recovery and restart. Run formatting, locked tests and strict
  all-targets Clippy. Record evidence and limitations in docs/playtests.md.
- Human observations of tactical usefulness and status comprehension remain
  required for experiential acceptance. Automated results cannot establish them;
  report outstanding human checks explicitly in the PR.

## Alternatives considered

Immediate recovery while occupied can sustain camping and weakens the ticket's
purpose. A fixed all-or-nothing cooldown is simple but produces an abrupt refill
and makes short useful returns less flexible. Continuous recovery after sustained
absence preserves partially useful visits and gives readable progress.
