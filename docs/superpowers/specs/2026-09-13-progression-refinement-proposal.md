# DRO-31 refinement proposal — not approved for implementation

This is parallel refinement requested while implementing DRO-30. It does not
change progression. The user has not approved four slots, milestone timing,
exclusions, ranks, or new upgrade content. DRO-22 owns expansion to twelve cards.

## Current behavior

`src/upgrades.rs` awards four XP per kill. Six level costs are 50, 75, 100, 125,
150 and 175 XP: 675 cumulative, or 169 kills without the optional 30-XP pickup
(162 with it). Every card is unique, but unchosen cards return and all six can be
acquired. Exhaustion already hides further promised rewards; Skip, stable offers,
input isolation, pause and restart are implemented. Earlier experience documents
with ten XP per kill and a three-minute run are historical, not current tuning.

DRO-30's current six-card charger comparison still lets successful unlimited
control camps finish all six by roughly three minutes. Arena changes alone do
not create an opportunity cost. Ordinary finite chargers and actual player
choices make timings vary substantially.

## Options

1. **Recommend: four irreversible upgrade opportunities, paced across the run.**
   Keep the six existing cards and their benefits/drawbacks. A completed run can
   acquire at most four, so taking one explicitly rules out another final benefit.
   This supports flexible combinations without authoring replacement cards.
2. **Mutually exclusive branches.** Pair mobility/armor or fire/support choices.
   This makes exclusions immediately understandable, but forces categories onto
   the current small catalog and can eliminate interesting hybrid builds.
3. **Upgrade ranks.** Spending another choice on an owned upgrade competes with
   breadth and offers more late decisions. This needs new rank balance/content,
   magnifies multiplicative drawbacks, and overlaps the later catalog task.

Slower XP alone delays all-benefit builds but does not prevent them; it should
support the chosen rule rather than be the only change.

## Concrete candidate rules for review

Offer four opportunities, with earliest active times 0, 75, 150 and 225 seconds
and cumulative earned XP requirements 50, 150, 350 and 600. Both requirements
must be met. Bank XP beyond a threshold and show the next opportunity's XP/time
requirements honestly; no modal appears early. Resolve at most one offer at a
time, using the existing stable three-card selection and fresh input rules.

Taking a card locks it for the encounter. Skip spends that opportunity, as Skip
already spends the current level; a skipped card may reappear at a later
opportunity. Therefore skipping can leave fewer than four final upgrades. After
four resolutions, or no eligible cards, show Build complete with selected cards
and no further level prompts or filler reward. Reset restores four opportunities.
XP can remain an internal statistic after completion, but must not promise a
fifth reward. No replacement/respec within a run is proposed.

These particular thresholds are starting points, not approved tuning. Without
the optional pickup, steady kill-rate models give the following earliest times:

| Kills/second | Opportunity 1 | Opportunity 2 | Opportunity 3 | Opportunity 4 |
| --- | ---: | ---: | ---: | ---: |
| 0.3 | 43s | 127s | 293s | Not reached in 5min |
| 0.6 | 22s | 75s | 150s | 250s |
| 1.2 | 11s | 75s | 150s | 225s |

Times use whole kills (13, 38, 88, 150). Real waves, combat, pickup collection,
skips and deaths do not follow a steady-rate model. A weak run may not finish a
build, but continues to have a useful pending decision late in play. A fast run
cannot consume every decision before the final phase.

Candidate builds to compare include Interceptor + Agile frame + Heavy rounds +
Rapid shield, versus Heavy armor + Heavy rounds + Rapid shield + Wide-area rockets.
These are test candidates, not demonstrated viable builds. Require ordinary
resource use and actual damage/handling measurements before claiming viability.

## Decisions needed before implementation

- Accept four total opportunities and the loss of an opportunity when skipping,
  or prefer four acquired-card slots with additional later chances after Skip?
- Accept explicit time gates, or prefer XP-only pacing despite greater kill-rate
  variance and the possibility of early completion?
- Confirm that the six existing cards remain the whole scope until DRO-22.

Then simulate low/typical/high kill-rate runs and two contrasting builds on the
final DRO-30 arena; record chosen and rejected cards, choice times, skips,
completion and survival. Human observations must show that the trade-off is
understood and that late decisions matter. Keep pause/reset/held-input/empty-pool
regressions, and do not label the issue solved from rate arithmetic alone.
