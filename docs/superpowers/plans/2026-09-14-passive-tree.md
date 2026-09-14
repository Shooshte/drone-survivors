# Permanent passive tree implementation plan

> Execute inline with the executing-plans skill; request an independent code review before delivery.

**Goal:** Add the agreed nine-node, five-rank session-persistent passive tree and hub purchase screen.
**Architecture:** Pure rank/purchase rules in passives.rs; campaign owns ranks; pristine baseline -> permanent -> temporary modifiers; mission input owns shopping; separate presentation and native fixture.
**Tech stack:** Existing Rust/Bevy 0.19.1, no new dependencies.

## Constraints

Follow docs/superpowers/specs/2026-09-14-passive-tree-design.md. Preserve direct-combat fixtures and resource settlement. Use this isolated worktree and commit each tested increment.

## Task 1: Deterministic purchases and effects
- [ ] Add passives.rs, passives/tests.rs and main module declaration. Test rejecting locked FireRate, buying ProjectileDamage with exactly 10 salvage, unlocking FireRate at first rank, five successive ranks, insufficient components/salvage rollback, capped purchase rollback, and maximal wallet balances.
- [ ] Observe failing tests, implement NodeId, PassiveTree with private [u8;9], rank(), next_cost(), availability(), purchase(node, wallet), and permanent modifiers. Validate actual outcomes from each stat including fire interval division, range/lifetime, upward contact-damage rounding and reserve factor.
- [ ] Run focused tests and commit pure rules.

## Task 2: Runtime lifecycle and energy flow
- [ ] Add Campaign.passives. In upgrades/runtime.rs, derive pristine+permanent tuning at reset and temporary choice; retain pristine startup baseline. Test purchases before launch, XP HeavyRounds/Interceptor composition, restart, success/failure, hub and repeated launches with no accumulation.
- [ ] Add EnergyConfig.reserve_cost and FlowConfig.reserve_cost (default 1). Test delivery with factor 0.75: 25 energy costs 18.75 reserve; 15 reserve delivers 20 energy, then stops. Include active modules, full battery, overlapping sources, and long/split updates. Observe failure before modifying flow's depletion span and reserve debit.
- [ ] Run full tests and commit lifecycle/effects.

## Task 3: Purchase screen and native evidence
- [ ] Add GamePhase::Passives and MissionAction::Passives/Purchase(NodeId). Gate purchases to passive phase, accept one action per press/release, handle 1–9 and Backspace. Test held/stale/multiple buttons, unaffordable actions, maxed state, and phase restrictions.
- [ ] Add passives/scene.rs and hub entry using existing visual style. Render three columns with three node cards; show rank, current/next percent, next price and prerequisite/purchase result. Verify phase visibility and native layouts at both supported sizes.
- [ ] Add opt-in native passive validation fixture and deterministic combat comparison tests with fixed enemies and no optional modules. Record measured base/upgraded results and native observations, with synthetic overrides disclosed.
- [ ] Run cargo fmt --check, cargo test --locked, cargo clippy --all-targets -- -D warnings, cargo dev fixtures and manual navigation. Request independent review, fix actionable findings, commit, push, create PR to main, and move Linear to In Review.
