# DRO-31 Implementation Plan

> Execute inline in the existing isolated worktree, using executing-plans and test-driven-development; request independent review before delivery.

**Goal:** Make XP purchases compete for four irreversible opportunities.
**Architecture:** Keep bounded accounting and deterministic sampling in UpgradeRun,
existing runtime input/lifecycle integration, and budget/progress copy in scene.
**Tech stack:** Rust, Bevy 0.19.1, existing native validation harness.

## Global constraints

Four total opportunities. Skip spends one. XP only; no time gates. Six existing
cards and effects. No arena, flight, wave, charger, module or reward changes.
Cumulative thresholds 50/200/500/1000 are provisional. Preserve input isolation,
pause, terminal precedence, exhausted-pool safety and reset.

## 1. Bounded accounting and integration

Files: src/upgrades.rs, src/upgrades/tests.rs, src/combat/upgrade_tests.rs,
src/combat/validation.rs, src/combat/validation_tests.rs,
src/combat/camping_tests.rs.

- [x] Add failing tests: award u32::MAX, resolve four alternating picks/skips,
  award more, and assert no fifth offer or pending choice. Check incremental
  threshold boundaries, banked XP, immediate completion, and deterministic reset.
- [x] Run `cargo test --locked upgrades::tests` and confirm behavioral failures.
- [x] Replace unbounded arithmetic with at most four costs [50,150,300,500].
  Keep earned level and carried XP; add resolved count and saturating total XP.
  Increment resolved only on a valid resolution; clear pending/mark exhausted
  at four. Keep pool exhaustion and sampling unchanged.
- [x] Adapt two-choice fixtures from 140 to 215 XP and the second cost from 75
  to 150. Use total XP for diagnostic accounting. Exercise four queued choices
  through actual keyboard/mouse input and restart after completion.
- [x] Run focused rule/runtime tests; commit the passing accounting change.

## 2. Player-facing opportunity costs

Files: src/upgrades/scene.rs, README.md.

- [x] Test opportunity count, permanent Skip consequence, queued fourth reward
  without a fifth XP target, and completion with an empty or partial build.
- [x] Show unresolved opportunity budget separately from earned pending count.
  Preserve card benefit/drawback copy and acquired build names. Update README
  to the approved rules. Expand the explicit native preview to all four queued
  choices (1,000 synthetic XP) to inspect final-opportunity and completion UI.
- [x] Run scene tests and inspect the native 640x480 overlay, normal overlay,
  queued state and completion. Commit presentation and documentation.

## 3. Pacing evidence and final review

Files: src/upgrades/tests.rs, src/combat/camping_tests.rs,
docs/playtests/dro-31-progression.md, docs/playtests.md.

- [x] Measure 0.3/0.6/1.2 kills per second with/without pickup and an authored
  maximum-kill envelope. Record actual choice times, not inferred viability.
- [x] Run finite-charger combat comparisons for different upgrade priorities and
  movement/power tactics. Record offers, picks, skips, completion, damage,
  resources and outcomes; state limits if candidates fail.
- [x] Run cargo fmt --check, cargo test --locked, and
  cargo clippy --all-targets --locked -- -D warnings.
- [x] Request independent review, fix material findings, rerun affected checks.
- Publishing after verification: commit evidence, push branch, open PR against main and update Linear with
  implemented rules, checks, evidence and any pending human acceptance.
