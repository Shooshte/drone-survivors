# Arena flight room implementation plan

> **For agentic workers:** Use subagent-driven-development or executing-plans to implement and review each bounded task. The user explicitly requested parallel agents and separate worktrees.

**Goal:** Give the scout useful high-speed flight room without shrinking visible threats.

**Architecture:** Shared authored layout coordinates feed geometry, navigation and validation. A following orthographic camera retains viewing scale; responsive status panels and projection-based indicators preserve navigation.

**Tech Stack:** Rust, Bevy 0.19.1, existing Cargo/nightly setup.

## Global constraints

- Floor 1920×1080, ceiling 300, chargers x = ±560.
- Keep established scout/enemy stats, modules, XP and wave schedule.
- Retain electrical shortcut and empty-battery detour.
- Camera minimum width 1120 and minimum height 800; fixed orientation.
- Verify 1120×720 and 640×480, pause/outcomes/restart.
- Human handling/pressure observations must be identified as pending when absent.

## Task 1: Arena layout and flight measurements

Files: `src/arena.rs`, `src/arena/scene.rs` grid only, `src/world/geometry.rs`,
`src/world/navigation.rs`, `src/world/scene.rs`, `src/energy.rs`, route/charger
validation pilots and geometry-dependent tests.

- [ ] Add failing clear-lane/curve measurements with production flight integration.
- [ ] Run focused tests and record the old arena boundary failures.
- [ ] Centralize charger/route coordinates in `src/world/layout.rs`; use
  `CHARGER_X: f32 = 560.`, `DETOUR_Z: f32 = 410.` and divider x = 120.
- [ ] Expand `Arena::default().half_size` to `Vec3::new(960., 150., 540.)`,
  extend divider and reposition low cover. Update navigation, dots and fixtures
  together; never alter enemy steering or physical tuning.
- [ ] Run world/route/charger/flight tests; measure clear lanes, curves and routes.
- [ ] Commit the layout and regression coverage.

## Task 2: Following camera and navigation indicators (parallel worktree)

Files: new `src/arena/camera.rs`, `src/arena/scene.rs` camera setup only,
`src/arena.rs` module export, `src/arena/tests.rs` camera tests.

- [ ] Test camera follows X/Z, fixed scale/orientation, freeze and restart.
- [ ] Replace whole-arena fit with `ScalingMode::AutoMin { min_width: 1120.,
  min_height: 800. }`, translate using scout ground position in presentation.
- [ ] Test projection-based edge placement at multiple aspect ratios and target
  directions, then add charger and spawn-warning labels from live entities.
- [ ] Hide visible targets, expired warnings and choice-overlay indicators;
  avoid HUD bands and duplicate warning labels in a direction.
- [ ] Run focused tests and commit camera/indicator changes.

## Task 3: Compact status panels (parallel worktree after refinement)

Files: `src/arena/scene.rs` HUD only, `src/combat/scene.rs`,
`src/energy/scene.rs`, `src/modules/scene.rs`, `src/world/scene.rs` HUD only,
`src/upgrades/scene.rs` HUD only.

- [ ] Inspect existing HUD resources and preserve every required state/feedback.
- [ ] Lay out shallow top/bottom bands; use readable text at minimum window size.
- [ ] Shorten repeated instructions and condense module statuses without hiding
  shield recharge, reserve delay/recovery or activation failures.
- [ ] Verify existing UI state tests and inspect native captures after integration.
- [ ] Commit presentation changes.

## Task 4: Integration, pressure, review and delivery

- [ ] Cherry-pick worktree commits; resolve only bounded shared-file overlap.
- [ ] Run `cargo fmt --check`, `cargo test --locked`,
  `cargo clippy --locked --all-targets -- -D warnings`.
- [ ] Run native routes/chargers and capture normal + 640×480 gameplay.
- [ ] Compare deterministic encounter measurements against the original arena;
  record pressure and fixture limitations in `docs/playtests/dro-30-arena.md`.
- [ ] Independently review full diff and fix actionable findings with regressions.
- [ ] Update README/playtest log, commit, push branch and open PR against `main`.
- [ ] Deliver DRO-31 refinement separately; do not implement unapproved rules.
