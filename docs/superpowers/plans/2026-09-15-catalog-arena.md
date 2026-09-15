# Catalog arena implementation plan

**Goal:** Deliver DRO-34 as the independently usable first PR of the DRO-22 split.
**Architecture:** Opt-in catalog module inside combat; production reset boundary
and gameplay plugins; a separate selector UI module and focused integration tests.
**Tech stack:** Rust, Bevy 0.19.1, native macOS, existing Cargo configuration.

## Constraints

- Worktree `.worktrees/dro-34-catalog-arena`, branch `codex/dro-34-catalog-arena`.
- No campaign introduction, save access, synthetic rewards or gameplay shortcuts.
- Campaign human acceptance remains pending.

## 1. Route and reset the isolated arena

- [ ] Add `src/combat/catalog.rs` and `src/combat/catalog/tests.rs`; register the
  module in `src/combat.rs`. Add a failing integration test proving `--validate
  catalog` is accepted, selecting a slot and launching preserves that loadout,
  and returning from a modified paused run restores production state.
- [ ] Extend `ValidationMode`/parser/install in `src/combat/validation.rs`, with
  catalog duration defaulting to 60 seconds and an early-return installer.
- [ ] Implement `CatalogArena` (scenario index, loadout, selecting flag, duration),
  `CatalogAction` (Previous, Next, CycleSlot(usize), Launch, Return), and
  `CatalogPlugin`. Use Transition for controls/boundary flags, Reset completion for
  selector phase, and production WaveConfig for the three scenario definitions.
- [ ] Run focused tests with `cargo test --locked catalog`; verify full reset,
  invalid/duplicate input, selection pause and absence of campaign state. Commit.

## 2. Present and exercise the selector

- [ ] Add `src/combat/catalog/scene.rs` for scenario and slot buttons, Launch,
  in-round Return, dynamic labels and visibility. Use the same CatalogAction for
  mouse and keyboard paths. Keep layout responsive within 640x480.
- [ ] Exercise real button interactions in tests; test held controls across launch
  and return, and wave spawning after the warning period. Commit working UI.
- [ ] Run `cargo dev -- --validate catalog` at 640x480 and 1120x720, exercise
  selection/loadout/launch/restart/return, inspect native screenshots and correct
  layout or lifecycle defects.

## 3. Verify and deliver

- [ ] Update README and docs/playtests.md with controls, isolation, baseline and
  final results, native evidence and limitations.
- [ ] Run `cargo fmt --check`, `cargo test --locked`, and
  `cargo clippy --all-targets --locked -- -D warnings`.
- [ ] Review the branch against the spec, fix actionable findings and commit.
- [ ] Push and open PR against main; link it from DRO-34 and keep DRO-22 pending.
  Dependent child PRs follow the arena merge rather than duplicating its changes
  in every PR against main.
