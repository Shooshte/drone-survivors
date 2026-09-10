# Combat Prototype Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Make the current integrated arena a repeatable five-minute survival prototype with rapidly accelerating enemy numbers and useful validation evidence.

**Architecture:** Keep enemy logic and player balance untouched. Generate authored bursts in WaveConfig; derive phase/lull presentation from the same phase definitions. Count spawn outcomes in a run-local resource nested in Encounter, and expose them to opt-in validation reporting.

**Tech Stack:** Rust, Bevy 0.19.1, controlled-time ECS tests, native macOS validation.

## Global Constraints

- Survival requires 300 seconds of active gameplay; upgrade-selection pauses do not count.
- Difficulty escalation comes only from increasing enemy numbers.
- Enemy health, contact damage, speed, acceleration, turning, pursuit, separation, navigation, targeting rules, and hazard interaction do not scale or otherwise change for difficulty.
- Preserve the existing arena, player stats, module and XP tuning for the initial implementation.
- Preserve cap 30, warning 0.75 seconds, clearance 120, cancellation, reservation and hitch rules.
- Human evidence is required to pass the gate; automation cannot establish perceived difficulty or two successful human tactics.
- Worktree: /Users/shooshte/projects/drone-survivors/.worktrees/dro-12-combat-prototype. Base: 08dd355, branch codex/dro-12-combat-prototype.

## Task 1: Authored waves, truthful HUD, and spawn accounting

**Files:** src/combat/waves.rs, src/combat/scene.rs, src/combat/wave_tests.rs, relevant combat integration tests.

**Interfaces:** Add WaveConfig phase/status helpers; Encounter owns `pub spawns: SpawnCounts`. SpawnCounts derives Default/Debug/Clone/Copy and has public usize fields: requested, admitted, rejected_cap, rejected_space, skipped_hitch, skipped_terminal, activated, cancelled. Review refinement: skipped_terminal accounts due requests suppressed by death/survival without creating warnings. Requested includes every due authored enemy; admitted counts created warnings; skipped_hitch counts enemies in older discarded bursts. While playing: requested = admitted + rejected_cap + rejected_space + skipped_hitch + skipped_terminal. Admitted = activated + cancelled + currently pending warnings. Terminal cancellation counts too; reset clears all fields.

- [x] Change the existing default-duration integration test to require survival at 300 seconds, prove Playing at 180, cancel pending warnings on completion, and reset run state. Run the focused test and observe its failure before implementation.
- [x] Generate warning times using integer seconds, with the opening at 3/18/33 and subsequent windows below. Suppress final six seconds of each pressure window. Keep phase metadata authoritative for HUD labels; do not infer lulls by calendar minute.

```rust
let mut bursts = vec![(3., 3), (18., 3), (33., 3)];
for (start, end, size, interval) in [(45,105,3,12), (105,165,4,8), (165,225,6,6), (225,300,8,4)] {
    for at in (start..end - 6).step_by(interval) {
        bursts.push((at as f64, size));
    }
}
```

- [x] Test window boundaries through real warnings, full warning delay, fixed enemy health/flight configuration across stages, no duplicate warnings, and lulls before non-minute boundaries. Cover accounting for admission, cap rejection, no safe position, cancelled warning, hitch, terminal cancellation, and reset.
- [x] Update duration-dependent fixtures to use configured duration. Preserve the old scripted-pilot victory test as an explicitly named legacy three-minute empty-arena fixture; do not weaken it or claim it proves the new scenario. Extend general full-run test budgets to 310 active seconds where they use the new default.
- [x] Run focused and full tests. Commit gameplay milestone with only task-owned files.

## Task 2: Opt-in observations and native validation

**Files:** src/combat/validation.rs, new focused observation module/tests if needed; README.md; docs/playtests.md.

**Interfaces:** Consume Encounter.spawns and existing WaveConfig/UpgradeRun/Energy state. Preserve `ValidationConfig { mode, seconds, enemies }`. Add Manual mode if needed for human record-only validation: no pilot, no synthetic XP, no automatic choices, no gameplay overrides. Existing automated modes remain explicitly labeled probes. Default encounter validation sample limit becomes 305 seconds; Stress and Choices retain their shorter defaults.

- [x] Add failing tests for a five-minute validation default and Manual mode preserving movement/module/choice input. Assert Manual does not alter waves, grant XP, refill hull, or drive the drone.
- [x] Record progress/final spawn counters and active-stage counts so a saturated cap is visible. Record module state changes, charger entry/exit, choices, and route side crossings without changing gameplay. Reset observations on R; do not count initial state as a deliberate decision.
- [x] Prevent paused intervals from entering frame-time samples on resume by retaining whether the previous sample boundary was Playing. Test a simulated long choice pause rather than relying on wall-clock sleeping.
- [x] Run a native baseline before wave edits, then the final normal/pilot and stress modes. Capture the actual scene at normal/minimum size and inspect the images. Document synthetic overrides and inability to infer human experience from probes. Do not raise the cap or adjust XP without evidence.
- [x] Update README to explain five-minute pacing and manual recording command. Record exact final schedule, actual results, and a repeatable human playtest sheet. Keep the gate pending when human evidence is unavailable.
- [x] Run cargo fmt --check, cargo test --locked, cargo clippy --all-targets --locked -- -D warnings. Commit validated observation/documentation milestone.

## Additional validated scope: footer readability

The baseline 640 × 480 native capture reproduces an overlap between hazard
instructions and XP summary: independent absolute nodes share bottom offsets of
108 and 112 pixels. Group the hazard summary, XP/acquired upgrades, and controls
in a shared footer flow, with compact text at minimum size. Keep critical state
and acquired names visible. This fixes a demonstrated acceptance failure within
the existing arena; it does not change game rules. Inspect native recaptures at
both sizes, including an acquired-upgrade list. Commit the presentation fix
separately and include it in whole-branch review.

## Task 3: Review and delivery

- [x] Review the complete branch against the approved spec and fix substantive findings with focused regressions.
- [x] Verify final tests, clean diff, native results, and truthful gate status. Commit any review fixes and evidence separately.
- [ ] Push codex/dro-12-combat-prototype and create a PR against main, with completed implementation and pending human gate evidence distinguished. Preserve the worktree for follow-up.

## Progress

- Baseline: all 175 tests pass on 08dd355. Spec approved by user's implementation request.

- Wave milestone committed 612461d; full 187tests pass.
- Footer milestone committed 005cf92; separate review clean and complete-scene native captures inspected.
- Combined formatting, 187tests, and strict all-targets Clippy passed during root verification.
- Native terrain Armored probe died at 89.307s; this does not establish a winning tactic. Human acceptance remains pending.

- Observation milestone committed b8addec; manual mode preserves player control and records decisions/population.
- Review fixes 772afc6 and 5d602fe cover terminal spawn accounting and immediate result reporting. Focused re-review found no remaining code issues.
- Final formatting, 190 tests, and strict all-targets Clippy pass at 5d602fe.
- Isolated full-scene cap-30 sample: p95 8.689 ms; earlier concurrent-build miss retained in docs/playtests.md.
- Implementation and evidence are ready for PR handoff; required human playtests remain pending.


## Approved minimal manual-playtest follow-up

The user approved the minimal scope: opening pacing and exhausted-upgrade UI.
Scout flight/arena changes and progression redesign move to Linear follow-ups.

1. In `src/combat/wave_tests.rs`, change the opening warnings to 3/13/23;
   first pressure window to 30–105 with 3 every8 seconds (9 bursts), opening lull
   to24, boundary to30, and total to46 bursts/262 requests. Update the real HUD
   boundary regression in `src/combat/feedback_tests.rs` to29/30. Run
   `cargo test --locked wave_tests` and confirm failure before tuning.
2. In `src/combat/waves.rs`, set opening end30/lull24/interval10 and first
   pressure start30/first_warning30/interval8. Preserve all later phases,
   300-second duration, cap30, safety, warnings, and enemy/player balance.
3. In `src/upgrades/tests.rs`, reproduce selecting the final eligible upgrade
   with no further pending level and require immediate exhaustion. In
   `src/upgrades/scene.rs`, cover complete-build HUD without XP/level promises.
   Run focused tests and confirm failures before changing their implementation.
4. In `UpgradeRun::prepare_offer`, check eligible-pool exhaustion even when
   pending==0, while preserving stable active offers and RNG. In `hud_copy`,
   return `BUILD COMPLETE | No more upgrades this run` and the acquired list
   when exhausted; otherwise preserve level/XP copy. Retain XP accounting,
   thresholds, six single-purchase upgrades, eligibility, and offer rules.
5. Run `cargo fmt --check`, `cargo test --locked --quiet`, and
   `cargo clippy --all-targets --locked -- -D warnings`, using the shared target
   directory. Review the focused diff, update README/checklist/playtest evidence,
   commit each implementation milestone, push the existing branch, and update PR11.
6. Create focused Linear tickets for scout handling, arena/Scout identity, and
   meaningful run progression, referencing existing DRO-22 for catalog expansion.
   Record their links in the handoff. Human acceptance stays pending another run.


## Approved camping-pressure and XP follow-up

The user approved larger late bursts, a repeatable real-arena camping check,
and a separate charger-depletion ticket, adding a requirement to lower kill XP
so upgrade choice timing remains roughly stable.

- Keep opening/Pressure I unchanged. Set the last three burst sizes to 6/9/12
  instead of 4/6/8, with existing intervals, lulls, cap 30 and warning/safety rules.
  The same 46 bursts request 375 enemies instead of 262.
- Add a small `ExperienceConfig { kill_xp: u32 }` resource in upgrades.rs,
  defaulting to 7, consumed by the runtime's existing kill-credit path.
  Pickup XP remains 30; thresholds and upgrade selection remain unchanged.
  Baseline potential kill XP: 262x10=2620. Proposed: 375x7=2625.
- First update wave-boundary/count and exact-once kill-credit regressions and
  observe failures. Then implement the two tuning changes and run focused tests.
- Add an opt-in deterministic balance probe under combat tests with default
  WorldGeometry, real charging/modules/damage/upgrades and controlled time.
  Compare old bursts/10 XP against new bursts/7 XP at both charger centers,
  using strong normal earned upgrade selections, overdrive-only and shield
  variants, plus a moving keyboard-pilot comparison. Starting at a charger is
  fixture setup; no subsequent position locking, HP overrides or synthetic XP.
  Record outcomes, choice times, spawn outcomes, peak population and energy.
  Treat this as a diagnostic, not human difficulty or performance proof.
- Use subagent-driven-development for the independent probe task while root
  implements tuning and the Linear handoff. Review all code before delivery.
- Inspect whether reduced XP delays the unchanged opening disproportionately.
  Document observed timing differences, and keep the experiential gate pending
  if stationary builds still win; charger depletion is not implemented here.
- Create the charger-depletion follow-up with finite delivery accounting,
  recovery, readable status, and pause/reset requirements; link it to DRO-12.
- Update README, spec/checklist and evidence; run formatting, full tests and
  strict Clippy, commit, push and update PR11. Preserve all flight/enemy rules.


XP refinement during verification: a global 7-XP candidate delayed the first
six left-charger choices by roughly 12/11/27/26/16/9 seconds. Retain 10 XP until
105 active seconds and reduce to 7 thereafter. ExperienceConfig exposes
`early_kill_xp`, `reduction_starts_at`, and `kill_xp`; reward uses kill time.
A regression covers the phase boundary, exact-once credit, and reset. Baseline
and final diagnostic scenarios must use the same module profiles at each charger.


## Midpoint-pressure opening refinement

Execute inline in the existing PR worktree with the user's tuning authorization.

- [x] Update existing wave regression expectations to 6/8/9/10/12 at intervals
  8/8/6/5/4, including real first-warning admission/activation, authored totals
  (50/497), phase boundaries, safe rejection and unchanged enemy stats. Run
  `cargo test --locked wave_tests` and confirm the old production values fail.
- [x] Apply the schedule in `src/combat/waves.rs`. In the XP exact-once/reset
  regression expect 4 XP for a kill before and after 105 seconds; update
  `ExperienceConfig` default early/late rewards to 4 after observing failure.
  Adjust mechanical reward assertions to the new default, keeping their original
  behavior coverage.
- [x] Freeze the immediately previous schedule explicitly in
  `src/combat/camping_tests.rs`; the tuned arm must use production defaults,
  including XP. Run `cargo test --locked camping_balance_probe -- --ignored
  --nocapture`, compare normal earned choices and pressure, and record both wins
  and losses. Do not alter outcome assertions to force a balance claim.
- [x] Run formatting, full locked tests and strict all-targets Clippy. Update
  README, playtest evidence/checklist, Linear DRO-12 and PR #11. Commit tuning and
  evidence separately, push, retain draft status and the human acceptance gate.


## Approved adjustment: one fewer enemy per burst

The user found the midpoint-pressure opening too intense and requested exactly
one fewer enemy in every wave. Change stage sizes 6/8/9/10/12 to **5/7/8/9/11**,
keeping all warning times, intervals, lulls and the 4 XP kill reward unchanged.
This removes 50 enemies from the 50-burst schedule: **447 requested total**.
Update existing count regressions and the current diagnostic expectation, run
formatting/locked tests/strict Clippy plus the opt-in comparison, record fresh
results, and commit/push to the existing PR. No other balance changes.
