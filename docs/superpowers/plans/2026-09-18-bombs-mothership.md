# Bombs and mothership implementation plan

> Execute coherent tasks with test-first checks and independent code review.

**Goal:** Deliver DRO-37 in a separate worktree, push and open a PR against main.
**Architecture:** Use ordered combat systems, shared spawn safety and catalog-only loadouts.
**Tech stack:** Rust, Bevy, existing native test/validation harness.

## Global constraints

Follow ../specs/2026-09-18-bombs-mothership-design.md. Keep campaign wave tables,
shop catalog, upgrade offers and save schema unchanged. Numeric values provisional.
No human playtest claim. Commit each coherent increment.

## Task 1: Bomb and Repulsor behavior

- [x] Add failing production regressions in src/combat/bomb_tests.rs.
- [x] Implement bombs.rs state, contact branch in lifecycle.rs, enum/config support
  in modules.rs and economy/runtime.rs; wire ordered systems in combat.rs.
- [x] Verify fuse, stacking, power/jam, shielding, resets and reward accounting with
  `cargo test --locked bomb`; commit behavior and tests.

## Task 2: Mothership admission and delivery

- [x] Add production tests in src/combat/mothership_tests.rs before implementation.
- [x] Implement mothership.rs cadence, parent-linked warnings and ring candidates;
  share waves.rs clearance and enforce parent liveness on activation.
- [x] Verify full warning, cap with reservations, no recursive spawn, wall/player
  clearance, source death, pause/reset and accounting; commit with integration.

## Task 3: Catalog presentation and native evidence

- [x] Add three catalog scenarios, Repulsor selection, cached visual cues and HUD.
- [x] Add opt-in bomb/mothership fixture, run native at both supported sizes,
  inspect captures and correct layout/readability issues.
- [x] Run `cargo fmt --check`, `cargo test --locked`,
  `cargo clippy --locked --all-targets -- -D warnings`, and `git diff --check`.
- [x] Obtain independent code review, fix findings and rerun affected checks.
- [x] Record commands/results/known limits in docs/playtests.md and dedicated
  evidence file. Commit, push codex/dro-37-bombs-mothership, open PR against main.


Implementation: `831863f`. Independent review found no actionable findings.
Verification: 447 tests passed, 5 existing probes ignored; format/strict Clippy
pass. Native fixtures cover both sizes and simultaneous bomb/jammer states.
Delivery: [PR #29](https://github.com/Shooshte/drone-survivors/pull/29) against main.
Branch and worktree are retained for review. Human acceptance remains pending.
