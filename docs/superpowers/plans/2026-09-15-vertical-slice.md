# Two-mission vertical-slice implementation plan

**Goal:** Demonstrate the first two existing missions through real rewards, purchases,
loadout changes and disk-backed save/resume, and fix the largest observed progression
issue. The user explicitly deferred human validation on September 15, 2026.

**Architecture:** Reuse MissionPlugin, the production combat/flight/world systems,
ModuleInventory and SavePlugin. Add an opt-in fixed-step campaign probe with ordinary
keyboard actions; keep synthetic evidence separate from natural gameplay results.
Keep mission 01 survival and mission 02 reconnaissance, placeholder geometry and the
bounded catalog. No campaign schema change is expected.

**Tech stack:** Rust, Bevy 0.19.1, existing Cargo/native macOS setup.

## Constraints

- Work only in `.worktrees/dro-21-vertical-slice`, branch `codex/dro-21-vertical-slice`.
- Test saves live in temporary directories; never modify the user's campaign.
- Agent runs cannot establish uncoached human usability, enjoyment or perceived power.
- The later human gate still blocks content expansion.

## 1. Measure the existing loop

- [ ] Run `cargo test --locked` on the unchanged baseline.
- [ ] Add `src/mission/slice_tests.rs`, registered as a test module in `src/mission.rs`.
  Start fresh disk-backed campaigns with production geometry, time, combat, upgrades,
  missions and save plugins. Use fixed 1/30-second updates and keyboard controls.
- [ ] Complete mission 01 without granting funds/health/XP or disabling waves; log
  elapsed time, kills, hull, loot and naturally earned upgrades.
- [ ] Use actual shop keyboard actions to buy/equip contrasting affordable modules,
  recreate the app from its saved file, and fly mission 02 through all sites and exit.
- [ ] Record failures as well as successful strategies. Identify the largest concrete
  pacing/progression issue before changing production behavior.

## 2. Fix and verify the measured issue

- [ ] Add a failing regression test for the chosen issue.
- [ ] Implement the smallest fix in the relevant mission/shop UI or rules module.
- [ ] Verify the regression, preservation of replay/retry/selection and save behavior,
  and repeat both campaign probes. Commit the verified behavior change.

## 3. Native presentation, evidence and delivery

- [ ] Exercise the changed loop presentation with `cargo dev` at 1120x720 and 640x480.
  Record any fixture setup explicitly; inspect screenshots and text bounds.
- [ ] Run `cargo fmt --check`, `cargo test --locked`, and
  `cargo clippy --all-targets --locked -- -D warnings`.
- [ ] Review the full branch, address actionable findings, and record outcomes,
  loadout/route rationale, save checks and evidence limits in
  `docs/playtests/dro-21-vertical-slice.md` and `docs/playtests.md`.
- [ ] Commit evidence, push the branch, open a PR against main and link it in Linear.
  Keep the deferred human checklist visibly pending.
