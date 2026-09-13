# DRO-31 progression evidence — September 13, 2026

## Approved rules and status

The user approved four total opportunities, Skip consuming an opportunity, and
XP-only progression. Implemented cumulative thresholds are **50 / 200 / 500 /
1,000 XP** (costs 50 / 150 / 300 / 500). Kill rewards remain 4 XP; the one-use
pickup remains 30 XP. There are no time gates, replacement, respec, branches,
ranks, additional cards, or changes to the six existing effects.

Taking any benefit spends one of four opportunities, so at least two of the six
benefits remain unavailable in a completed build. Skip spends the same budget
without acquiring a card. Unchosen/skipped cards can return within the remaining
budget. Normal deterministic offer sampling and module prerequisites remain.
Four resolutions immediately finish progression even when some/all were skipped;
pool exhaustion also completes immediately. Internal XP remains a statistic;
there are no fifth-reward promises or filler prompts.

Implementation and native mechanical/UI checks are complete. **Human acceptance
is pending:** these observations do not establish that players understand the
trade-off or that two tactics are viable. The four scripted candidates below all
died; retain those failures rather than treating late choices as proof of fun or
balance. XP pacing is a provisional starting point for human playtests.

## XP pacing

`cargo test --locked progression -- --include-ignored --nocapture`

[Full measured output](dro-31-progression-results.txt). The steady-rate model
credits whole kills at 30 Hz until just before the 300-second terminal boundary.
The optional pickup is credited at time zero solely to bound its possible effect;
a real player must fly to it. Times below are active seconds, rounded to tenths.

| Kills/s | Pickup | Choice 1 | Choice 2 | Choice 3 | Choice 4 |
| --- | --- | ---: | ---: | ---: | ---: |
| 0.3 | No | 43.3 | 166.7 | Unreached | Unreached |
| 0.3 | Yes | 16.7 | 143.3 | Unreached | Unreached |
| 0.6 | No | 21.7 | 83.3 | 208.3 | Unreached |
| 0.6 | Yes | 8.3 | 71.7 | 196.7 | Unreached |
| 1.2 | No | 10.8 | 41.7 | 104.2 | 208.3 |
| 1.2 | Yes | 4.2 | 35.8 | 98.3 | 202.5 |

A separate authored-wave upper bound assumes every requested enemy is admitted
and dies immediately at activation. Without the pickup, choice times are
**19.75 / 62.75 / 135.75 / 225.75**; with it, **3.75 / 54.75 / 129.75 / 215.75**.
Those are bounds under this encounter schedule, not promised playtest timings.
They show useful progression cannot be fully consumed in the opening even at
ideal kill speed, without an elapsed-time gate. Slower runs still have an
unearned useful choice available, but may never reach it before death/survival.
Zero kills plus the pickup is insufficient for a first choice.

## Actual arena, finite chargers, ordinary damage and earned XP

`cargo test --locked progression_balance_probe -- --ignored --nocapture`

Uses the existing 30 Hz arena harness and normal keyboard choice controls. No
health, XP, damage, position, or charger-reserve grants are made in these four
cases. The moving pilot leaves overdrive enabled; the relay pilot cycles the two
central chargers while conserving power between visits. Priorities select from
actual stable offers and fall back to other existing cards; desired builds are
not forced into the offer pool. All unchosen offers are recorded in the raw log.

| Priority / tactic | Outcome / active seconds | Kills | Picks at active seconds |
| --- | --- | ---: | --- |
| Mobility / moving | Dead 45.9 | 24 | Agile frame 13.7 |
| Mobility / relay | Dead 111.3 | 82 | Agile frame 26.7; Rapid shield 72.5 |
| Armor / moving | Dead 84.0 | 62 | Heavy rounds 13.7; Rapid shield 63.3 |
| Armor / relay | Dead 282.2 | 386 | Heavy rounds 26.7; Rapid shield 73.3; Heavy armor 143.4; Wide-area rockets 228.5 |

The armor relay reached its final choice during final pressure. That final offer
contained Wide-area rockets, Agile frame, and Interceptor: taking the rockets
permanently excluded the offered mobility benefits from this run. It finished
with four upgrades, 55 energy and zero hull. It visited the two central fields
13/12 times, drew approximately 642/688 energy, and powered overdrive for 137.5s.
Peak living enemies were 28; of 414 requested spawns, 413 were admitted and one
was rejected by the population cap. No space rejection or hitch skipping occurred.

The mobility relay visited fields 5/5 times, drew 233.3/283.7 energy, and powered
overdrive for 55.7s. Both moving pilots exhausted their starting energy after
10s of overdrive and never visited a charger. Their early deaths do not isolate
upgrade balance from poor resource use or the fixed pilot's limitations.
The relay fixture primarily exercises overdrive; shield/rocket upgrade picks
therefore do not demonstrate effective powered use of those modules. All four
results are mechanical diagnostics, not two viable human tactics.

## Native observation

Used `cargo dev -- --validate choices --seconds 600`. The explicit preview now
starts with 1,000 synthetic XP to expose four queued opportunities; ordinary
runs never receive that grant. This is a UI fixture, not a balance playtest.
The CLI executable was also copied into a temporary local macOS app bundle so
computer-use tools could address it. The bundle ran the same built executable,
assets, and flags with no gameplay modifications.

- At **640 × 480**, all three cards, budget, permanent Pick/Skip consequence,
  Skip control and reset instructions fit without clipping.
- Mouse-picked Heavy rounds at opportunity 1; exactly one choice was consumed.
- Backspace skipped opportunity 2; opportunity 3 retained two remaining choices.
- Keyboard-picked Rapid shield; the next modal said **FINAL OPPORTUNITY** with
  one remaining opportunity. Gameplay stayed at active time zero throughout.
- Resized to **1120 × 720** and inspected the final modal. Budget and all
  benefits/drawbacks remained readable. Fixed the small singular-copy issue
  discovered there (`1 choices ready` → `1 choice ready`).
- Picked Heavy armor. Modal closed, gameplay resumed, hull became 150, and
  **BUILD COMPLETE** listed Heavy rounds, Rapid shield, Heavy armor. No XP or
  further-choice promise remained. One Skip correctly left only three upgrades.
- Resized back to **640 × 480**; completed build names and status remained readable.
- Pressed R: baseline hull 100, shield drain 8, no acquired upgrades, four
  opportunities, and XP 0/50 returned. No synthetic XP was regranted by restart.

[Native fixture log](dro-31-native-ui.txt). The saved capture shows the first naturally earned offer after reset (one
choice ready), rather than the initial four-choice synthetic queue. It is
1280 × 960 physical pixels for a 640 × 480 logical Retina window:

![Four-opportunity choice preview](../images/dro-31-choices-640x480.png)

## Automated checks and review

New coverage includes bounded maximal XP awards, every threshold boundary,
four picks/skips, immediate completion, excess XP, no fifth reward, HUD budget
and Skip text, final-opportunity copy, four queued keyboard resolutions without
movement/module/time leakage, queued held-mouse release/repress, and reset after
completion restoring deterministic offers. Existing effect, prerequisites,
pause, empty-pool, terminal-precedence and runtime checks remain passing.

An independent read-only review found no material implementation defect. It
identified missing queued-mouse coverage; that regression was added and passes.
Final checks: `cargo fmt --check` passes; `cargo test --locked` passes **247 tests**
with four opt-in diagnostics excluded; `cargo test --locked -- --ignored` passes
**all four diagnostics**; `cargo clippy --all-targets --locked -- -D warnings`
passes. The final native executable also builds with locked dynamic linking.

Evidence review caught a missing model-asset link and an overwritten log in the
initial temporary app bundle. Added the asset link and repeated the entire
native sequence with the final executable and visible model. The committed log
is the successful repeat (no asset errors, all four choices recorded), and the
screenshot was refreshed from that repeat.

## Required human follow-up

Play normal five-minute encounters with at least two contrasting tactics. Record
all offers, choices/Skip times, acquired and excluded benefits, module use,
charging/route decisions, death/survival, and examples of a drawback changing an
action. Ask players to explain what their final choice prevented them from taking
and whether an unreached XP threshold felt achievable. Establish two viable
tactics and meaningful late decisions before closing experiential acceptance.
Do not infer those outcomes from this synthetic UI fixture or failed pilots.
