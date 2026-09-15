# Campaign Save and Resume Implementation Plan

**Goal:** Complete DRO-18 with persistent campaign progress and explicit recovery.
**Architecture:** Validated versioned snapshot, atomic local storage, and a Bevy
persistence plugin installed only for normal play. Loading restores hub state;
autosave runs after campaign mutations and result settlement.
**Tech stack:** Rust, Bevy 0.19.1, serde/serde_json, tempfile, native filesystem.

## Constraints

One native save slot. No combat serialization. Preserve unreadable saves. Never
settle a loaded reward again. New campaign replacement is confirmed. Fixtures do
not touch real saves. Worktree `.worktrees/dro-18-save-resume`, branch
`codex/dro-18-save-resume`, PR base `main`.

## Tasks

- [ ] Storage: add failing round-trip, malformed/version/invariant and atomic
  replacement/conflict tests in `src/save/tests.rs`; run `cargo test save`.
  Implement `Snapshot::capture`, `Snapshot::restore`, `Store::read` and
  `Store::write` in `src/save/{snapshot,storage}.rs`. Add narrow validated restore
  constructors to passive tree/module inventory. Run tests and commit.
- [ ] Runtime/UI: add failing ECS tests in `src/save/runtime_tests.rs` using the
  existing mission test harness. Implement startup Continue/New/confirm/retry
  states and autosave after `GameplaySet::Completion`, before presentation.
  Block further mission input on save errors. Preserve the source phase on retry.
  Add a separate overlay in `src/save/scene.rs`, normal-launch installation in
  `src/main.rs`, and hub shortcut. Run tests and commit.
- [ ] Verify: exercise real native UI with an isolated `DRONE_SAVE_PATH`, restart
  the executable, inspect restored values and recovery/cancel at 640x480.
  Run `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets`.
  Obtain independent code review, address actionable findings, update README and
  `docs/playtests.md`, commit, push, open PR against main and update Linear.
