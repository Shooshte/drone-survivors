## Outcome and scope

Chunk 6 of 24 in the Drone Survivors development work plan. **Phase A — Prove tactical combat.**

Add run-local XP and paused upgrade choices to the existing three-minute arena. Choices should visibly change how the drone flies, survives, and fights, with drawbacks that affect tactics. Use six single-purchase upgrades for this prototype.

**Depends on:** [DRO-7: Swarms and combat feedback](<https://linear.app/drone-survivors/issue/DRO-7/03-swarms-and-combat-feedback>), [DRO-9: Four-slot module controls](<https://linear.app/drone-survivors/issue/DRO-9/05-four-slot-module-controls>).

Keep the current encounter, four-slot loadout, normal controls, and charging fields. Permanent progression, shops/loadout editing, campaign persistence, additional drone classes, and the full twelve-upgrade catalog remain downstream work.

## Agreed upgrade catalog

The catalog and trade-offs below were refined with the user. Values are initial balance targets, subject to playtesting.

| Upgrade | Benefit | Drawback |
| -- | -- | -- |
| Interceptor | +30% horizontal acceleration and maximum horizontal speed | Battery capacity falls from 100 to 75 |
| Agile frame | +40% turn, tilt, and automatic leveling response | Maximum hull falls from 100 to 80 |
| Heavy armor | +50 maximum hull | −25% acceleration in every movement direction |
| Heavy rounds | Double basic projectile damage: 10 → 20 | −10% acceleration in every movement direction |
| Rapid shield | Powered shield recharge falls from 5s to 2.5s | Shield drain rises from 8 to 12 energy/s |
| Wide-area rockets | Explosion radius grows from 70 to 105 world units | Rocket launch interval increases from 2s to 3s |

Heavy rounds preserves the basic firing interval and adds no energy cost. Its damage benefit affects basic projectiles only; weapon overdrive still modifies their firing rate. Heavy armor and Heavy rounds penalties both cover horizontal movement, climbing, descending, and braking. They affect translational acceleration, not turn/tilt/leveling response or speed caps directly. Preserve neutral level hover and arena containment; do not implement the penalty by reducing rotor lift alone and causing unavoidable sinking. Enemy flight is unchanged.

## Eligibility, composition, and applying effects

* Each definition can be selected once per run. Different upgrades can combine; unselected cards remain eligible for later offers.
* Rapid shield and Wide-area rockets require their corresponding module to be equipped in any slot. A module can be switched off and still qualify.
* Flight, hull, battery, and basic weapon modifiers remain active for the rest of the run. Powered module benefits and drain changes operate under the existing module power rules.
* Derive effective stats from baseline configuration and the selected upgrade set. Multiply percentage modifiers on the same stat; add flat hull changes. Selection order must not change final maximum stats. For example, Heavy armor plus Heavy rounds gives 0.75 × 0.90 = 67.5% baseline acceleration; Heavy armor plus Agile frame gives 130 maximum hull.
* Increasing maximum hull adds the same amount to current hull; decreasing it clamps current hull to the new maximum. Lower battery capacity clamps stored energy without refilling it. Immediate current hull can depend on when damage and choices occur.
* Apply the selected upgrade once, before the next gameplay step. Preserve module ON/OFF state, shield blocks, and normalized cooldown/recharge progress when durations change; selection must not grant a free shot or shield block. Already launched projectiles retain their launch-time properties.

## XP and pacing — provisional implementation defaults

These details are proposed starting rules, rather than playtest results:

* Start at level 1 with zero XP. Award 10 XP per confirmed enemy death, once, whether killed by a basic projectile or rocket splash. Credit every distinct kill in a multi-kill.
* Place one clearly visible exploration pickup at a fixed, reachable location away from the player spawn and charging fields. It grants 30 XP, is collected using a visible 3D proximity radius, disappears immediately, and returns on restart. No recurring spawn or XP magnet system in this chunk.
* First level-up costs 50 XP; each subsequent level-up costs 25 more than the previous one: 50, 75, 100, 125, and so on. Subtract each earned threshold and retain excess XP.
* Target roughly three to five choices during an actively played three-minute run. Verify reachability and pacing through playtests and tune the centralized XP values if needed.
* Multiple thresholds crossed by one update queue multiple earned choices. Present one choice at a time, recomputing eligibility after each selection.

## Choice UI and pause contract

* Sample up to three distinct eligible upgrades without replacement for each offer. Keep the offer stable until resolved; randomness must be reproducible in rule tests.
* Show the upgrade name, exact benefit, exact drawback, and that the effect lasts for this run. Display fewer cards with clear copy when fewer than three are eligible.
* Always offer an explicit **Skip** button when at least one card is shown, including offers with fewer than three cards. The player may decline every offered upgrade. Skipping consumes this earned choice without applying a benefit or drawback, refunding XP, banking a choice, or rerolling the current offer. The level remains earned and skipped cards remain eligible for future offers. Advance to the next queued choice, or resume gameplay when none remain.
* Support Skip by mouse and a labeled fresh Backspace press, with the same opening-frame and held-input protections as card selection.
* When no upgrades remain eligible, show a brief nonblocking explanation and resume automatically. Consume pending choices, suppress further empty dialogs, and retain XP accounting; exhaustion must never trap the player or repeatedly interrupt play.
* Freeze all gameplay during selection: player/enemy movement, damage, projectiles and lifetimes, spawning and warnings, encounter time, energy drain, charging, shield recharge, weapon cooldowns, and protection timers. UI input remains responsive. Resuming must not catch up elapsed wall time or expire absolute-time deadlines.
* Support mouse selection and a fresh press of 1–3 for the displayed cards. Prevent module-toggle input from becoming a selection on the opening frame, selection from toggling modules on resume, and a held key/click from selecting the next queued offer.
* R restarts from selection and wins over other inputs; Escape retains its existing quit behavior.
* On a gameplay update that also ends the run, resolve death/survival before opening an upgrade choice. Preserve existing death/survival precedence. Never open choices over a terminal result.
* Show level and XP progress in the HUD plus a compact acquired-upgrade summary. Cards and updated hull/battery/module values remain readable at 640×480.

## Lifecycle and implementation boundaries

Restart clears XP, levels, pending/current offers, selected upgrades, and their effects; restores the exploration pickup and baseline hull/battery/module/flight settings; and performs the existing encounter reset. Terminal states freeze the run until restart.

Use typed Rust definitions and a focused run-local upgrade system. Separate XP accounting, eligibility/selection, and effective-stat calculation from presentation. Integrate with existing confirmed kills, gameplay scheduling, and reset behavior; avoid duplicating combat or flight logic. No campaign persistence or generic content editor is needed.

## Acceptance checks

- [ ] Basic kills, rocket multi-kills, and exploration grant XP exactly once; threshold boundaries, excess XP, and queued level-ups are correct.
- [ ] Offers contain only distinct eligible unowned upgrades; selected effects apply once; unselected effects do not apply. Cover three, two, one, and zero eligible definitions, including exhaustion with queued choices.
- [ ] Skip works by mouse and keyboard for any nonempty offer, changes no upgrade stats, consumes exactly one earned choice, preserves earned level/excess XP, and either advances the queue or resumes play. Skipped cards remain eligible later; held input cannot skip multiple choices.
- [ ] Verify every catalog benefit and drawback, including Heavy armor and Heavy rounds acceleration changes in horizontal and vertical movement and braking, Heavy rounds unchanged firing rate/free basic fire, and composition with powered mobility/overdrive.
- [ ] Verify effect composition, selection-order-independent maximum stats, current hull/battery adjustments, equipped-module eligibility, and progress-preserving cooldown/recharge changes.
- [ ] A long choice pause changes no gameplay state or remaining gameplay timer. Resume causes no catch-up movement, damage, firing burst, power drain, or spawn burst.
- [ ] Selection works with mouse and keyboard without held-input, queued-choice, or module-toggle leakage. Restart and terminal outcomes take precedence correctly.
- [ ] Restart restores baseline stats, XP, offers, pickup, and the existing encounter state, including from selection and after combined upgrades.
- [ ] Play two contrasting builds in the same encounter and record choice timing, noticeable benefits, and a concrete tactical decision changed by a drawback in `docs/playtests.md`. Include a 640×480 readability check.

## Definition of done

- [ ] The acceptance checks pass and the feature can be demonstrated independently of unfinished downstream systems.
- [ ] No known blocking defect in the changed behavior; record results, balance adjustments, and next steps in `docs/playtests.md`.
- [ ] For implementation, run `cargo fmt --check`, `cargo test --locked`, and `cargo clippy --all-targets --locked -- -D warnings`; use `cargo dev` for native gameplay checks. Test meaningful rules and edge cases rather than copies of static content.
- [ ] Review and commit this chunk separately.

## Shared context

Build a roughly 90-minute, 12-mission campaign where tactical decisions make the player feel clever and combat progression makes them feel powerful. Use Rust/Bevy and the existing Cargo setup. Initial target: native macOS with keyboard/mouse. Use reusable content and placeholders until the full loop is proven.

Source: `docs/superpowers/plans/2026-09-08-game-development-work-plan.md` (September 8, 2026). Issue refined September 9, 2026. This is an implementation specification; unchecked acceptance items do not represent completed work.