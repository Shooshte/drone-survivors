# DRO-12 camping and XP timing diagnostic

Recorded on 2026-09-10 at source commit `552d118`. This tuning was later superseded by [the midpoint-pressure opening](dro-12-midpoint-comparison.md).

Ignored 30 Hz Bevy simulation, run with `cargo test --locked camping_balance_probe -- --ignored --nocapture`. It uses `WorldGeometry::default()`, the real hazard, `ArenaPlugin`, `CombatPlugin`, `UpgradePlugin`, normal charging/modules, real damage, and earned XP. Each camp is staged once at `(±280, 90, 0)`; there are no later transform/velocity writes, health overrides, invulnerability, forced offers, forced selections, or synthetic XP. Modules and choices use ordinary guarded keyboard edges. The moving control uses `validation::pilot_keys`, a deterministic automated pilot rather than a human strategy.

Baseline uses late burst sizes 4/6/8 and 10 XP per kill. Candidate uses 6/9/12 and 10 XP per kill before 105 active seconds, then 7 XP. Estimated authored XP budgets are 2,620 and 2,733 respectively if kills occur in their spawn phase; the sampled camps realized those totals.

`spawns` is requested/admitted/rejected-cap/rejected-space/skipped-hitch/skipped-terminal/activated/cancelled.

| balance | tactic | outcome / active s | hull | kills / earned XP | peak enemies / warnings | spawns | energy | level/xp/pending |
|---|---|---:|---:|---:|---:|---|---:|---:|
| baseline | left overdrive | Survived / 300.000 | 130 | 262 / 2620 | 10 / 8 | 262/262/0/0/0/0/262/0 | 75.000 | 14/20/0 |
| baseline | right overdrive | Survived / 300.000 | 130 | 262 / 2620 | 11 / 8 | 262/262/0/0/0/0/262/0 | 75.000 | 14/20/0 |
| baseline | left overdrive+shield | Survived / 300.000 | 130 | 262 / 2620 | 10 / 8 | 262/262/0/0/0/0/262/0 | 75.000 | 14/20/0 |
| baseline | right overdrive+shield | Survived / 300.000 | 130 | 262 / 2620 | 11 / 8 | 262/262/0/0/0/0/262/0 | 75.000 | 14/20/0 |
| baseline | moving pilot | Dead / 169.067 | 0 | 65 / 650 | 6 / 6 | 70/70/0/0/0/0/67/3 | 14.167 | 6/150/0 |
| candidate | left overdrive | Survived / 300.000 | 130 | 375 / 2733 | 14 / 12 | 375/375/0/0/0/0/375/0 | 75.000 | 14/133/0 |
| candidate | right overdrive | Survived / 300.000 | 130 | 375 / 2733 | 15 / 12 | 375/375/0/0/0/0/375/0 | 75.000 | 14/133/0 |
| candidate | left overdrive+shield | Survived / 300.000 | 130 | 375 / 2733 | 14 / 12 | 375/375/0/0/0/0/375/0 | 75.000 | 14/133/0 |
| candidate | right overdrive+shield | Survived / 300.000 | 130 | 375 / 2733 | 15 / 12 | 375/375/0/0/0/0/375/0 | 75.000 | 14/133/0 |
| candidate | moving pilot | Dead / 181.233 | 0 | 99 / 801 | 10 / 9 | 105/105/0/0/0/0/102/3 | 0.000 | 7/126/0 |

No run collected the exploration pickup. All admitted camp enemies activated and died; every spawn-accounting identity held. Both chargers stayed naturally stationary to within 0.001 units with velocity below 0.001. The shield variants exactly matched their same-node overdrive-only counterparts in this deterministic run.

## Choice records

- Baseline left, both module profiles: `15.167 L2 HeavyRounds; 39.100 L3 RapidShield; 63.700 L4 HeavyArmor; 96.900 L5 Interceptor; 132.767 L6 AgileFrame; 168.900 L7 WideAreaRockets`.
- Candidate left, both module profiles: `15.167 L2 HeavyRounds; 39.100 L3 RapidShield; 63.700 L4 HeavyArmor; 96.900 L5 Interceptor; 130.600 L6 AgileFrame; 166.967 L7 WideAreaRockets`.
- Baseline right, both module profiles: `16.967 L2 HeavyRounds; 39.133 L3 RapidShield; 65.533 L4 HeavyArmor; 97.100 L5 Interceptor; 130.600 L6 AgileFrame; 168.400 L7 WideAreaRockets`.
- Candidate right, both module profiles: `16.967 L2 HeavyRounds; 39.133 L3 RapidShield; 65.533 L4 HeavyArmor; 97.100 L5 Interceptor; 130.567 L6 AgileFrame; 167.033 L7 WideAreaRockets`.
- Baseline moving: `16.100 L2 HeavyRounds; 39.200 L3 RapidShield; 64.333 L4 HeavyArmor; 96.300 L5 Interceptor; 131.367 L6 AgileFrame`.
- Candidate moving: `16.100 L2 HeavyRounds; 39.200 L3 RapidShield; 64.333 L4 HeavyArmor; 96.300 L5 Interceptor; 130.967 L6 AgileFrame; 169.533 L7 WideAreaRockets`.

The first four choices match exactly. Candidate camp choice deltas are 0/0/0/0/-2.167/-1.933 seconds on the left and 0/0/0/0/-0.033/-1.367 on the right. The moving candidate's fifth choice is 0.400 seconds earlier and it earns a sixth before dying.

For the shield profiles, choice times match the same-node overdrive-only records above, but AgileFrame is selected fourth and Interceptor fifth. This corrects the originally combined selection-order description; the timings and outcomes are unchanged.

## Interpretation

The phased XP schedule preserves early upgrade timing and the moving probe survives longer, from 169.067 seconds and 65 kills to 181.233 seconds and 99 kills. Camping still clears the full candidate schedule at either charger, with or without shield. This leaves the camping gate pending the planned charger-depletion work; no production values were altered to force a different result.

An exploratory pre-refinement run with 7 XP from time zero also had both sampled camps survive and kill all 375 enemies. Its left-camp first-six timing deltas were +11.600/+10.800/+27.000/+25.700/+16.433/+9.233 seconds, and its moving control died at 79.000 seconds versus 169.067 baseline. This exploratory candidate was superseded by phased XP.
