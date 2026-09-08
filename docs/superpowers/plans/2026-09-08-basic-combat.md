# Basic Combat Implementation Plan

> Execute the approved design task-by-task, with test-first checks and review
> before delivery. User has authorized implementation, commits, push, and PR.

**Goal:** Deliver DRO-6's repeatable 3D combat encounter.

**Architecture:** Keep arena movement, combat rules, and presentation separate.
Order reset, movement, combat, and presentation explicitly. Use ECS tests with
controlled time and shared visual assets rather than a physics dependency.

**Tech Stack:** Rust nightly, Bevy 0.19.1, existing Cargo tooling, native macOS.

## Constraints

- Follow docs/superpowers/specs/2026-09-08-basic-combat-design.md.
- Preserve the existing movement controls and bounds.
- Treat 0.75-second invulnerability and other tuning as playtest starting points.
- Keep the pre-existing untracked campaign plan out of commits.

## Task 1: Chasing and projectile combat

Files: create src/game.rs, src/combat.rs, src/combat/collision.rs,
src/combat/tests.rs; modify src/main.rs and src/arena.rs.

Interfaces: GamePhase::{Playing, Dead}; GameplaySet::{Reset, Movement, Combat,
Presentation}; CombatConfig resource; Enemy/Projectile components;
PlayerHealth and Weapon resources; CombatPlugin.

- [ ] Add ECS tests for chase in three axes/bounds, nearest target/range/ties,
      straight shots, moving-target sweep, first impact, and projectile expiry.
- [ ] Run `cargo test --locked combat` and observe missing behavior.
- [ ] Implement shared phase/ordering, fixed fixture, chase, firing, and sweeps.
- [ ] Run `cargo test --locked`; preserve all existing flight tests.

Collision example to verify the rule, with normalized frame time:

```rust
assert_eq!(segment_sphere(Vec3::ZERO, Vec3::X * 100., Vec3::X * 50., 10.), Some(0.4));
assert_eq!(segment_sphere(Vec3::ZERO, Vec3::X * 100., Vec3::Y * 50., 10.), None);
```

## Task 2: Damage and encounter lifecycle

Files: extend src/combat.rs and src/combat/tests.rs; use the same phase/set
interface for movement gating and reset precedence.

- [ ] Add failing ECS tests: simultaneous contacts cause one hit, contact after
      the deadline causes another, separation prevents damage, death freezes
      gameplay, repeated R clears transients and restores the exact fixture.
- [ ] Implement player-wide invulnerability, saturating health, one-time death,
      and ordered reset. Destroyed enemies cannot cause contact damage.
- [ ] Run `cargo test --locked`; commit the verified combat rules.

## Task 3: Presentation and delivery

Files: create src/combat/scene.rs; modify src/arena/scene.rs, src/main.rs,
README.md, and docs/playtests.md.

- [ ] Add reusable enemy/projectile meshes and materials plus one persistent HUD.
- [ ] Verify repeated restart does not accumulate entities or visual assets.
- [ ] Render native gameplay and exercise available keyboard controls; record
      observation limits honestly.
- [ ] Run `cargo fmt --check`, `cargo test --locked`, and
      `cargo clippy --all-targets --locked -- -D warnings`.
- [ ] Request independent source review while completing the native smoke check;
      fix actionable findings and rerun affected checks.
- [ ] Commit presentation/docs, push codex/dro-6-basic-combat, and create a PR
      against main with summary, ticket link, validation, and playtest limits.
