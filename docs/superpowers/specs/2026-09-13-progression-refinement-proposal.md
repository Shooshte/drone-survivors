# DRO-31 refinement proposal — not approved for implementation

Originally prepared alongside DRO-30; revisited for DRO-31 on September 13 after
DRO-30 and DRO-33 merged. Implementation is isolated on
`codex/dro-31-progression`, branched from main at `f5a7bab`. This proposal does not
change progression. Four opportunities versus four acquired-card slots, Skip
semantics, and time gates remain awaiting the user's answers. Keep the six
existing cards and effects in scope; DRO-22 owns expansion to twelve cards.

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

## Refined acceptance and implementation boundaries

- Every offer states the remaining opportunity/slot budget and the consequence
  of Skip before the player acts. Taking a card is irreversible for that run.
- The HUD distinguishes insufficient XP from a future time gate, if adopted.
  Earned XP is banked; it never creates an early modal or an extra fifth benefit.
- Late-earned opportunities remain usable, one stable offer at a time. If
  multiple gates are already satisfied, held keys or clicks cannot resolve the
  next offer. Terminal outcomes take precedence even at a gate boundary.
- Completion is immediate after the final permitted resolution/acquisition or
  an empty eligible pool. Show the acquired build, with no further promised
  choices, empty dialogs, filler rewards, or recurring prompts.
- Keep kill/pickup rewards, upgrade effects and module prerequisites unchanged
  initially. Tune only progression requirements using recorded evidence. Do not
  alter waves, chargers, arena, flight, or author new cards/ranks in this issue.
- Add focused rule and runtime regression coverage for XP banking, applicable
  gate boundaries, Skip, stable offers, input isolation, pause, terminal
  precedence, completion, empty pools, and deterministic full restart.
- Record low/typical/high kill-rate timings with and without the pickup, plus
  real-arena observations for two contrasting build/tactic candidates. Separate
  arithmetic and scripted evidence from human playtest observations.
- Run `cargo fmt --check`, `cargo test --locked`, and
  `cargo clippy --all-targets --locked -- -D warnings`; inspect native choice and
  completion presentation at 640 × 480 and the normal window size.
- Human evidence must establish understandable opportunity costs and two viable
  tactics. If unavailable, open the requested PR with that acceptance explicitly
  pending rather than describing the ticket as solved.
