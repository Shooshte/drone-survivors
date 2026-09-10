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

**Interfaces:** Add WaveConfig phase/status helpers; Encounter owns `pub spawns: SpawnCounts`. SpawnCounts derives Default/Debug/Clone/Copy and has public usize fields: requested, admitted, rejected_cap, rejected_space, skipped_hitch, activated, cancelled. Requested includes every due authored enemy; admitted counts created warnings; skipped_hitch counts enemies in older discarded bursts. While playing: requested = admitted + rejected_cap + rejected_space + skipped_hitch. Admitted = activated + cancelled + currently pending warnings. Terminal cancellation counts too; reset clears all fields.

- [ ] Change the existing default-duration integration test to require survival at 300 seconds, prove Playing at 180, cancel pending warnings on completion, and reset run state. Run the focused test and observe its failure before implementation.
- [ ] Generate warning times using integer seconds, with the opening at 3/18/33 and subsequent windows below. Suppress final six seconds of each pressure window. Keep phase metadata authoritative for HUD labels; do not infer lulls by calendar minute.

```rust
let mut bursts = vec![(3., 3), (18., 3), (33., 3)];
for (start, end, size, interval) in [(45,105,3,12), (105,165,4,8), (165,225,6,6), (225,300,8,4)] {
    for at in (start..end - 6).step_by(interval) {
        bursts.push((at as f64, size));
    }
}
```

- [ ] Test window boundaries through real warnings, full warning delay, fixed enemy health/flight configuration across stages, no duplicate warnings, and lulls before non-minute boundaries. Cover accounting for admission, cap rejection, no safe position, cancelled warning, hitch, terminal cancellation, and reset.
- [ ] Update duration-dependent fixtures to use configured duration. Preserve the old scripted-pilot victory test as an explicitly named legacy three-minute empty-arena fixture; do not weaken it or claim it proves the new scenario. Extend general full-run test budgets to 310 active seconds where they use the new default.
- [ ] Run focused and full tests. Commit gameplay milestone with only task-owned files.

## Task 2: Opt-in observations and native validation

**Files:** src/combat/validation.rs, new focused observation module/tests if needed; README.md; docs/playtests.md.

**Interfaces:** Consume Encounter.spawns and existing WaveConfig/UpgradeRun/Energy state. Preserve `ValidationConfig { mode, seconds, enemies }`. Add Manual mode if needed for human record-only validation: no pilot, no synthetic XP, no automatic choices, no gameplay overrides. Existing automated modes remain explicitly labeled probes. Default encounter validation sample limit becomes 305 seconds; Stress and Choices retain their shorter defaults.

- [ ] Add failing tests for a five-minute validation default and Manual mode preserving movement/module/choice input. Assert Manual does not alter waves, grant XP, refill hull, or drive the drone.
- [ ] Record progress/final spawn counters and active-stage counts so a saturated cap is visible. Record module state changes, charger entry/exit, choices, and route side crossings without changing gameplay. Reset observations on R; do not count initial state as a deliberate decision.
- [ ] Prevent paused intervals from entering frame-time samples on resume by retaining whether the previous sample boundary was Playing. Test a simulated long choice pause rather than relying on wall-clock sleeping.
- [ ] Run a native baseline before wave edits, then the final normal/pilot and stress modes. Capture the actual scene at normal/minimum size and inspect the images. Document synthetic overrides and inability to infer human experience from probes. Do not raise the cap or adjust XP without evidence.
- [ ] Update README to explain five-minute pacing and manual recording command. Record exact final schedule, actual results, and a repeatable human playtest sheet. Keep the gate pending when human evidence is unavailable.
- [ ] Run cargo fmt --check, cargo test --locked, cargo clippy --all-targets --locked -- -D warnings. Commit validated observation/documentation milestone.

## Task 3: Review and delivery

- [ ] Review the complete branch against the approved spec and fix substantive findings with focused regressions.
- [ ] Verify final tests, clean diff, native results, and truthful gate status. Commit any review fixes and evidence separately.
- [ ] Push codex/dro-12-combat-prototype and create a PR against main, with completed implementation and pending human gate evidence distinguished. Preserve the worktree for follow-up.

## Progress

- Baseline: all 175 tests pass on 08dd355. Spec approved by user's implementation request.
