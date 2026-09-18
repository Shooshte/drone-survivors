# DRO-40: Twelve temporary upgrades

## Scope and authorization

DRO-40 is the next ready child of DRO-22. Linear records the user's September 15 approval of the six additions below and the four-opportunity limit. This task implements that approved design in the isolated catalog arena. The campaign keeps its six original definitions, progression, and save format. Numerical tuning remains provisional; campaign introduction and human acceptance remain gated.

## Rules

Retain the original six cards unchanged. Add:

| Card | Benefit | Drawback | Equipment |
| --- | --- | --- | --- |
| Efficient coils | 25% less drain for every module | Activation threshold +100% (10 to 20 energy) | At least one module |
| Reserve battery | 50% more battery capacity | 20% lower maximum horizontal flight speed | Any |
| Long-range rounds | 50% greater basic targeting range | Basic shot interval +25% | Any |
| Hot overdrive | 50% greater powered Overdrive firing multiplier | Overdrive drain +50% | Overdrive |
| Rapid repair | Double powered hull repair rate | Repair drain +50% | Repair |
| Wide repulsor | 50% greater pulse radius | Pulse interval +50% | Repulsor |

Activation uses the existing minimum-energy rule, not an additional energy debit. The card must call it an activation requirement. Capacity increases do not refill the battery. Multipliers compose from the captured baseline; individual cards apply once, independent of order. Efficient coils scales all six module drains, including other upgrade penalties. Maximum hull/capacity clamps and existing projectile payload behavior stay intact. Shield, rocket and Repulsor cooldown progress survive choices; basic fire already retimes on the first resumed frame. Restart/return clears all effects and run state.

## Arena interaction

Use a second setup page, reached with U or a button, so the existing scenario/module page remains readable at 640x480. Four preview slots cycle eligible, distinct cards using 1–4 or clicks; each row states benefit and drawback. Launch queues the chosen cards as ordinary choice modals using synthetic XP equal to their normal cumulative cost. Each Pick or Skip spends one of the same four opportunities. Empty preview slots leave the remaining opportunities earnable normally from the twelve-card pool. An empty preview grants no XP. Changing modules clears newly ineligible previews. Launch/restart can replay the configured preview; return clears active effects while preserving setup choices.

The preview queue belongs only to the catalog resource/run; default/campaign runs sample only the original pool. This avoids adding a parallel effect-application pipeline. Alternatives considered: hard-coded build presets cannot exercise every combination; adding twelve controls to the existing page overflows the minimum window. A focused second page and the normal choice pipeline keep both behavior and presentation bounded.

## Evidence

Behavior tests cover offer isolation/eligibility, duplicate prevention, four opportunities including skips, modifier composition, runtime effects, pause/cooldown continuity and complete reset. Arena tests exercise keyboard and click setup, module changes, preview launch, normal choices, restart and return. Native agent fixtures at 1120x720 and 640x480 inspect all cards and representative combinations with explicit synthetic XP/input labeling. Run cargo fmt --check, cargo test --locked and strict Clippy; document limits in docs/playtests.md. Agent checks are not human acceptance or a balance gate.
