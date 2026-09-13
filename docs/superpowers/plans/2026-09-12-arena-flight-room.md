# Arena flight room implementation plan

> **For agentic workers:** Use subagent-driven-development or executing-plans to implement and review each bounded task. The user explicitly requested parallel agents and separate worktrees.

**Goal:** Give the scout useful high-speed flight room without shrinking visible threats.

**Architecture:** Shared authored layout coordinates feed geometry, navigation and validation. A following orthographic camera retains viewing scale; responsive status panels and projection-based indicators preserve navigation.

**Tech Stack:** Rust, Bevy 0.19.1, existing Cargo/nightly setup.

## Global constraints

- Final floor 1920×3240, ceiling 300, six chargers x = ±560 and z = -1080, 0, +1080 (user refinement).
- Keep established scout/enemy stats, modules, XP and wave schedule.
- Retain electrical shortcut and empty-battery detour.
- Camera minimum width 1120 and minimum height 800; fixed orientation.
- Verify 1120×720 and 640×480, pause/outcomes/restart.
- Human handling/pressure observations must be identified as pending when absent.

## Task 1: Arena layout and flight measurements

Files: `src/arena.rs`, `src/arena/scene.rs` grid only, `src/world/geometry.rs`,
`src/world/navigation.rs`, `src/world/scene.rs`, `src/energy.rs`, route/charger
validation pilots and geometry-dependent tests.

- [x] Add failing clear-lane/curve measurements with production flight integration.
- [x] Run focused tests and record the old arena boundary failures.
- [x] Centralize charger/route coordinates in `src/world/layout.rs`; use
  `CHARGER_X: f32 = 560.`, `DETOUR_Z: f32 = 410.` and divider x = 120.
- [x] Expand `Arena::default().half_size` to `Vec3::new(960., 150., 1620.)`,
  extend divider and reposition low cover. Update navigation, dots and fixtures
  together; never alter enemy steering or physical tuning.
- [x] Run world/route/charger/flight tests; measure clear lanes, curves and routes.
- [x] Commit the layout and regression coverage.

## Task 2: Following camera and navigation indicators (parallel worktree)

Files: new `src/arena/camera.rs`, `src/arena/scene.rs` camera setup only,
`src/arena.rs` module export, `src/arena/tests.rs` camera tests.

- [x] Test camera follows X/Z, fixed scale/orientation, freeze and restart.
- [x] Replace whole-arena fit with `ScalingMode::AutoMin { min_width: 1120.,
  min_height: 800. }`, translate using scout ground position in presentation.
- [x] Test projection-based edge placement at multiple aspect ratios and target
  directions, then add charger and spawn-warning labels from live entities.
- [x] Hide visible targets, expired warnings and choice-overlay indicators;
  avoid HUD bands and duplicate warning labels in a direction.
- [x] Run focused tests and commit camera/indicator changes.

## Task 3: Compact status panels (parallel worktree after refinement)

Files: `src/arena/scene.rs` HUD only, `src/combat/scene.rs`,
`src/energy/scene.rs`, `src/modules/scene.rs`, `src/world/scene.rs` HUD only,
`src/upgrades/scene.rs` HUD only.

- [x] Inspect existing HUD resources and preserve every required state/feedback.
- [x] Lay out shallow top/bottom bands; use readable text at minimum window size.
- [x] Shorten repeated instructions and condense module statuses without hiding
  shield recharge, reserve delay/recovery or activation failures.
- [x] Verify existing UI state tests and inspect native captures after integration.
- [x] Commit presentation changes.

## Task 4: Integration, pressure, review and delivery

- [x] Cherry-pick worktree commits; resolve only bounded shared-file overlap.
- [x] Run `cargo fmt --check`, `cargo test --locked`,
  `cargo clippy --locked --all-targets -- -D warnings`.
- [x] Run native routes/chargers and capture normal + 640×480 gameplay.
- [x] Compare deterministic encounter measurements against the original arena;
  record pressure and fixture limitations in `docs/playtests/dro-30-arena.md`.
- [x] Independently review full diff and fix actionable findings with regressions.
- [x] Update README/playtest log and commit validation evidence.
- Delivery: push `codex/dro-30-arena` and open PR against `main` after final review.
- [x] Deliver DRO-31 refinement separately; do not implement unapproved rules.

## User refinement: longer floor, six chargers, more spawn points

- [x] Triple floor north–south length to 3240, extend north divider, retain detour.
- [x] Add named NW/NE and SW/SE charging pairs, with independent reserves/reset.
- [x] Present six compact statuses and group off-screen charger indicators.
- [x] Increase perimeter samples along long sides; derive candidate count/search budget.
- [x] Update multi-node fixtures; run extended-lane, route, spawn, recharge and UI checks.
- [x] Repeat native captures and pressure comparison against final arena.
