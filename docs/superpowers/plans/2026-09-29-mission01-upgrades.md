# Mission 01 combined upgrades implementation plan

**Goal:** Test the compact payload mission and twelve existing upgrades together.
**Architecture:** Enable the expanded offer pool only from Mission01 context;
retain catalog previews only in the catalog. Extend existing campaign shop and
save serialization for the two already implemented support modules.
**Tech:** Rust/Bevy; shared Cargo target; existing native and headless fixtures.

## Task1: support-module campaign integration

Owned files: src/modules.rs, src/modules/shop*.rs and tests, src/save/* plus
legacy fixture expectations that explicitly assumed four purchasable modules.

- [ ] Red tests: purchase both modules for15 each, six-row keyboard wrap, owned
  and equipped support modules round-trip, legacy saves still accepted.
- [ ] Add campaign catalog including both modules, serialize enum variants,
  remove catalog-only purchase gating for these, update shop copy/layout.
- [ ] Keep four loadout slots and Mission01 reserved fourth slot; do not turn
  six-module iteration into six-slot assignment in existing fixtures.
- [ ] Focused tests and compact native shop check; commit bounded changes.

## Task2: Mission01 expanded offers and combined validation

Owned files: src/upgrades/runtime.rs, src/mission upgrade integration tests,
src/mission/blockout_validation.rs, README and playtest evidence.

- [ ] Red full-plugin test: launchMission01 no freeXP; enumerate all eligible
  cards via earned offers; support prerequisite and four-choice/reset checks.
- [ ] Select expanded offers when BlockoutRun.enabled or catalog flag is true,
  without enabling catalog previews or modifying global pool during baseline.
- [ ] Assert switching back to ordinary mission uses original six, preserving
  catalog behavior and applying effects from pristine+permanent baseline.
- [ ] Extend opt-in native Mission01 fixture for fundedsupportloadout and
  controlled earnedXP/normalchoiceinput; inspect shop/choice/missionbothsizes.
- [ ] Full tests/fmt/strictClippy, independent review, docs/commits/pushPR34.
