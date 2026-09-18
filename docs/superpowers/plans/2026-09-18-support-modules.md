# Support Modules Implementation Plan

> **For agentic workers:** Use subagent-driven-development for the bounded gameplay task and requesting-code-review for the final branch review.

**Goal:** Complete Repair and Repulsor in the isolated combat catalog for DRO-39.

**Architecture:** Extend ModuleConfig and the existing staged PowerFrame. Apply support effects in combat order using actual power delivery and the existing pulse/bomb state; present all six modules through catalog-only selection.

**Tech Stack:** Rust, Bevy 0.19.1, existing native fixtures and Rust tests.

## Global Constraints

- Four equipped slots and four upgrade opportunities per run.
- Repair: 6 hull/s, 12 energy/s, clamp to effective maximum, no full-hull banking or revival.
- Repulsor: 8 energy/s, 2s interval, 180-unit 3D radius, 260-unit/s outward impulse, no damage; solid terrain blocks reach.
- Existing activation threshold is 10 energy, not an extra debit.
- Catalog-only types must stay excluded from campaign shops, loadouts and serde saves.
- Human acceptance and campaign introduction remain gated.

### Task 1: Gameplay and regression tests

Files: src/modules.rs, src/energy.rs, src/combat.rs, src/combat/bombs.rs, new focused support behavior/test file(s), exhaustive matches in src/modules/shop.rs and shop_preview.rs.

- [ ] Add tests first for repair power accounting/cap/banking/pause/jam/reset/terminal behavior and repulsor range/cover/no-damage/physics/bomb behavior. Observe failures before implementation.
- [ ] Add Repair as a serde-skipped variant; retain campaign ALL and the default loadout. Add config tuning for both support modules.
- [ ] Carry paid repair duration through PowerFrame; accumulate fractional hull safely after damage resolution, clearing credit at full hull and boundaries.
- [ ] Extend the single existing Repulsor pulse to apply bounded outward enemy velocity, preserving collision-aware movement and bomb precedence.
- [ ] Verify targeted tests and save/shop rejection, then commit the coherent gameplay increment.

Tests use production plugins with manual Time, actual energy::prepare/update and normal phase boundaries. For one second powered Repair from hull 50, expect hull 56 and battery 88. For 0.5 powered seconds before depletion, expect 3 hull and disabled modules. Full-hull elapsed time must never become future healing. An in-range enemy must get outward velocity with unchanged hull, encounter kills and player rewards; an occluded or out-of-range enemy must not.

### Task 2: Catalog selection, presentation and native fixture

Files: src/combat/catalog.rs, src/combat/catalog/scene.rs, src/modules/scene.rs, src/combat/ordnance_scene.rs, src/combat/catalog_tests.rs, new src/combat/catalog/module_validation.rs.

- [ ] Add a failing selector integration test proving all six distinct modules can be selected and duplicate types are skipped.
- [ ] Append Repair after the existing Repulsor choice to preserve existing scenario fixtures. Add concise descriptions and configured drain/activation values to selected slot labels; show repair status and configurable pulse radius.
- [ ] Add DRONE_MODULE_SMOKE native fixture exercising real slot selection, power use, repair, pulse, bomb dislodge, lock, pause and reset. Explicitly label synthetic damage/positions/XP.
- [ ] Run native fixture with DRONE_CAPTURE_DIR at both supported sizes; inspect saved screenshots and text bounds. Commit presentation and fixture.

### Task 3: Verification, review and delivery

Files: README.md, docs/playtests.md, docs/playtests/dro-39-support-modules.md and selected evidence.

- [ ] Run `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings`.
- [ ] Record test/native evidence and limits; request independent code review, resolve actionable findings and rerun affected checks.
- [ ] Commit documentation; push codex/dro-39-repair-repulsor and open PR against main with DRO-39 link and verification evidence.
