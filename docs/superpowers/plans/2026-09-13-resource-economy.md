# Resource Economy Implementation Plan

> **For agentic workers:** Use executing-plans for the integrated implementation and requesting-code-review for independent review. Track the checkboxes below.

**Goal:** Implement DRO-14 pickups, session wallet, once-only mission rewards and clear UI.

**Architecture:** Keep accounting pure; attach attempt pickups to mission reset/cleanup; settle inside existing mission finalization. Keep visuals separate and reuse current UI.

**Tech Stack:** Rust, Bevy 0.19.1, existing Cargo dependencies.

## Global constraints

Use the approved spec in `docs/superpowers/specs/2026-09-13-resource-economy-design.md`. No new dependencies, enemy types, shop, save system or handcrafted missions. Preserve legacy direct-combat validation. All changes belong to this worktree/branch.

## Task 1: Accounting and terminal settlement

Files: create `src/economy.rs`, `src/economy/tests.rs`; modify `src/main.rs`, `src/mission.rs`; add `src/mission/economy_tests.rs`.

Interfaces: `Amounts { salvage: u64, components: u64 }`; `try_credit` and `try_spend` return `Result<(), TransactionError>` atomically; `RewardReceipt::settle(collected: Amounts, succeeded: bool, wallet: &mut Amounts)` returns a stable receipt with collected/lost/bonus/credited/balance/error. `AttemptResources` holds collected amounts. Campaign owns wallet and result owns receipt.

- [x] Write failing rounding and atomic transaction tests, then run `cargo test --locked economy` and inspect the failure.
- [x] Implement checked credit/debit and payout calculation, including zero, partial remainder and overflow rollback. Success example 19/3 -> 29/4, failure -> 4/0.
- [x] Add terminal/replay/restart tests using `mission::tests::{app,launch,tick}` before wiring settlement into `finalize` and resetting attempt collection at mission boundaries.
- [x] Run focused tests, format, inspect diff and commit accounting/lifecycle.

## Task 2: Drops, caches and attraction

Files: `src/economy/runtime.rs`, `src/economy/runtime_tests.rs`, `src/combat/feedback.rs`, `src/combat.rs`, `src/arena.rs`, `src/game.rs`.

Interfaces: `Pickup { amount: Amounts, radius: f32, attracted: bool }`, optional `ComponentCache`; `EconomyConfig` contains chaser `DropRule` and three cache positions. Consume `CombatOutcome::Hit { entity, position, killed }` as current Chaser facts, preserving combat behavior. `LootRng` and an attempt death set guarantee repeatable once-only rolls. Add `GameplaySet::Collection` between Combat and Progression.

- [x] Write failing tests for controlled drop outcomes, deduplication, collection boundaries/LOS, no expiry, pause and terminal/reset precedence.
- [x] Generate grounded pickups from kill facts after combat; move attracted salvage toward Scout with a bounded step, collect cache immediately in range. Prevent collection through solid terrain.
- [x] Spawn/reset three reachable caches; cleanup all pickups on terminal/hub transitions. Keep collected amounts for the result only until settlement; preserve banked wallet.
- [x] Test real bullet/rocket/hazard death feeds, cache reachability and repeated mission cycles. Run focused tests and commit.

## Task 3: Presentation and native validation

Files: `src/economy/scene.rs`, `src/mission/scene.rs`, `src/mission/validation.rs`, `src/main.rs`, `README.md`.

- [x] Add presentation tests for hub balances, immutable result breakdown and zero counters after restart.
- [x] Create reusable salvage/caches meshes and materials; attach an unbanked row beneath combat HUD with existing responsive font behavior.
- [x] Show both currencies as short result sections (collected, lost, bonus, banked, total); expose banking errors. Add accurate briefing collection/payout guidance without overcrowding the minimum window.
- [x] Extend native mission validation to capture nonzero rewards and reset/replay behavior, disclosing synthetic inputs; run native captures at both supported sizes, inspect images and correct layout issues.
- [x] Update README and commit presentation/validation.

## Task 4: Review and delivery

- [x] Run `cargo fmt --check`, `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings`.
- [x] Review with requesting-code-review subagent while performing native verification. Fix findings with focused regressions and verify affected checks.
- [x] Record actual evidence/limitations in `docs/playtests.md` and an issue-specific playtest report; commit.
- [x] Latest main rechecked (0 upstream commits missing), branch pushed, and [PR #18](https://github.com/Shooshte/drone-survivors/pull/18) opened against main. Worktree preserved.

## Progress

- Baseline: 262 tests passed, 4 opt-in diagnostics ignored, from fresh origin/main 975fd8f.

- Implementation complete in 4739002, 1e30492 and 19e404e.
- Final validation: 280 tests pass, 4 existing opt-in diagnostics ignored; formatting and strict all-target Clippy pass.
- Independent accounting, runtime and whole-branch reviews approved. Added repeatable-sequence regression from minor runtime review feedback.
- Both native UI sizes pass. Ordinary mouse/keyboard run collected 1 salvage and naturally failed at 0:37/18 kills; failure paid 0/0 and hub retained exactly one completion. Replay/R restored collection and gameplay.
