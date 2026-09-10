# DRO-12 midpoint-pressure opening comparison

Recorded on 2026-09-10 at source commit `f159e56`. Supersedes the tuning in
[the earlier camping comparison](dro-12-camping-comparison.md).

Command: `cargo test --locked camping_balance_probe -- --ignored --nocapture`.
The previous arm freezes the 375-enemy schedule and 10 XP before 105 seconds /
7 XP afterward. The tuned arm uses production defaults: 497 enemies, with
6/8/9/10/12 per burst at 8/8/6/5/4-second intervals, and 4 XP throughout.

The ten deterministic 30 Hz simulations use the real world geometry, hazard,
charging, damage, modules and normally earned guarded upgrade selections. Camp
fixtures are staged once at either charger; no subsequent position/velocity,
health, invulnerability or XP overrides are applied. The moving keyboard pilot
is a diagnostic, not a human tactic. Choice pauses do not count toward active time.

## Outcomes

| Balance | Tactic | Outcome / active seconds | Hull | Kills / earned kill XP | Peak enemies / warnings | Final energy |
| --- | --- | --- | --- | --- | --- | --- |
| baseline | camp-left-overdrive | Survived / 300.000 | 130 | 375 / 2733 | 14 / 12 | 75.000 |
| baseline | camp-right-overdrive | Survived / 300.000 | 130 | 375 / 2733 | 15 / 12 | 75.000 |
| baseline | camp-left-overdrive-shield | Survived / 300.000 | 130 | 375 / 2733 | 14 / 12 | 75.000 |
| baseline | camp-right-overdrive-shield | Survived / 300.000 | 130 | 375 / 2733 | 15 / 12 | 75.000 |
| baseline | moving-keyboard-pilot | Dead / 181.233 | 0 | 99 / 801 | 10 / 9 | 0.000 |
| tuned | camp-left-overdrive | Survived / 300.000 | 130 | 497 / 1988 | 15 / 12 | 75.000 |
| tuned | camp-right-overdrive | Survived / 300.000 | 130 | 497 / 1988 | 15 / 12 | 75.000 |
| tuned | camp-left-overdrive-shield | Survived / 300.000 | 130 | 497 / 1988 | 15 / 12 | 75.000 |
| tuned | camp-right-overdrive-shield | Survived / 300.000 | 130 | 497 / 1988 | 15 / 12 | 75.000 |
| tuned | moving-keyboard-pilot | Dead / 134.000 | 0 | 127 / 508 | 9 / 9 | 0.000 |

No run collected the pickup. All camps stayed naturally stationary and admitted,
activated and killed every requested enemy (375 previous / 497 tuned). Cap/space
rejection, skipped spawns and camping cancellations were zero. In the moving
pair, previous requested/admitted 105, activated 102 and cancelled 3; tuned
requested/admitted 135, activated 128 and cancelled 7. There were no rejections
or skipped spawns. Both terminal accounting identities passed for every case.

## Choice times

Times below are active seconds; both module profiles match at each charger.

| Balance / tactic | Choices 1–6 |
| --- | --- |
| baseline / camp-left-overdrive | 15.167 / 39.100 / 63.700 / 96.900 / 130.600 / 166.967 |
| baseline / camp-right-overdrive | 16.967 / 39.133 / 65.533 / 97.100 / 130.567 / 167.033 |
| baseline / moving-keyboard-pilot | 16.100 / 39.200 / 64.333 / 96.300 / 130.967 / 169.533 |
| tuned / camp-left-overdrive | 20.333 / 42.067 / 66.267 / 98.033 / 127.200 / 157.000 |
| tuned / camp-right-overdrive | 20.367 / 41.533 / 66.100 / 97.533 / 127.767 / 157.500 |
| tuned / moving-keyboard-pilot | 21.833 / 42.667 / 72.133 / 99.167 / 132.567 |

Overdrive-only and moving selection order is HeavyRounds, RapidShield,
HeavyArmor, Interceptor, AgileFrame, WideAreaRockets (the tuned moving pilot dies
before the sixth). Shield profiles exchange Interceptor and AgileFrame at
choices four and five. All choices came from normal offers.

Compared with the previous tuning, camping choices are within 9.967 seconds
(first choices 3.400–5.167 seconds later; sixth choices 9.533–9.967 seconds
earlier). The moving pilot's five shared choices are within 7.800 seconds. This
roughly preserves upgrade pacing for the sampled tactics without retaining the
old early reward of 10 XP. Total kill XP is lower after the catalog is exhausted;
matching post-exhaustion XP was not the target.

## Interpretation and verification

The opening now supplies the former midpoint's six enemies every eight seconds.
Scheduled active arrival rates rise 45/60/90/120/180 per minute before lulls and
spawn limits. The moving pilot dies at 134.000 seconds instead of 181.233,
despite killing 127 enemies instead of 99. This is evidence of earlier pressure
in this scripted case, not proof of newcomer difficulty or fun.

All charger camps still survive. More scheduled enemies do not resolve unlimited
stationary sustain; [DRO-32 charger depletion](https://linear.app/drone-survivors/issue/DRO-32/temporarily-deplete-charging-zones-to-prevent-unlimited-stationary)
and the human experiential gate remain pending.

Verification: 196 default tests passed, 0 failed, 1 ignored. The ignored probe was
explicitly run and passed all ten scenarios. Formatting, strict all-targets
Clippy and focused code review passed. These are headless results; they do not
replace the earlier source-specific native rendering measurements.
