# Mission 01 Playtest Tuning Implementation Plan

> Use subagent-driven-development for bounded combat implementation/review and integrate independent pickup/presentation work in this session.

**Goal:** Make the compact payload mission legible and active, with easier scrap collection and rising carrying pressure.
**Architecture:** Preserve mission-local isolation through BlockoutRun. Existing enemy flight, spawn warnings, collection, holdout and settlement remain authoritative; extend their tuning and presentation.
**Tech Stack:** Rust, Bevy0.19.1, existing headless/native test harnesses.

## Global constraints

User-approved design: `docs/superpowers/specs/2026-09-27-mission01-playtest-tuning.md`.
Keep scale12, Scout420u/s, slowdown0.6, exact fixed groups, cap96, fair bounded
queues, pause/reset, source indestructibility and existing save format. Other
missions/catalog retain old tuning. No new mission timer failure.

## Task 1 — Encounter pressure and payload escalation

Files: src/combat/mission01.rs, src/combat/enemies.rs, new focused test module if needed.
Consume BlockoutRun.enabled and ObjectiveRun.ready(). Existing SpawnWarning uses
`ready_at: Encounter.elapsed + WaveConfig.warning_seconds`; activation remains in
waves::update. Expose any new diagnostic counters only as needed by tests.

- [x] Add failing cases for pre-pickup patrol admission/activation, 5/4/3/2-second source tiers, phase freeze/reset, bounded hitch demand and fair admission under saturation.
- [x] Introduce patrol queue at3s/every8s, four chasers, safe current-player-relative candidate sites550–850u away; use normal warnings and cap accounting.
- [x] Track total carrying elapsed separately from current source interval credit; coalesce backlog and retain exact fixed queues.
- [x] Apply340u/s to Mission01 Chaser flight, preserving other enemy variants/maps.
- [x] Run targeted tests and real stationary/moving pressure probes; review and commit own files.

## Task 2 — Mission-local salvage attraction

Files: src/economy/runtime.rs, src/economy/runtime_tests.rs.
Consume optional BlockoutRun and PlayerPath. Cache logic remains unchanged.

- [x] Add failing tests for attraction at horizontal300 and flight height, rejection outside300, swept flybys, walls and legacy100u behavior.
- [x] Use mission-local horizontal proximity for ordinary salvage. At each swept contact, evaluate actual3D clear sight; continue ordinary900u/s collection motion.
- [x] Run economy and lifecycle tests; preserve one-time credit and pause/cleanup.

## Task 3 — Holdout readability and final playtest

Files: src/mission/blockout.rs, src/mission/blockout_scene.rs, src/mission/blockout_validation.rs, src/mission/objectives.rs, README.md, docs/playtests/dro-42-mission01-blockout.md.

- [x] Set HOLDOUT_RADIUS840; convert geometry-independent timing fixtures to use the constant.
- [x] Add persistent optional reward/instruction text on map; nearby/active prompt shows stay30s, +5components, exit warning, countdown/progress and terminal status. Use constants for timing/reward copy.
- [x] Add scene state coverage for approach, entry, pause, completion, forfeit and map switch; native captures check both window sizes.
- [x] Run full locked suite, fmt and strict Clippy; inspect native captures and document tuning/probe limits.
- [x] Independent whole-diff review, address findings, commit final evidence and report playtest launch instructions.
