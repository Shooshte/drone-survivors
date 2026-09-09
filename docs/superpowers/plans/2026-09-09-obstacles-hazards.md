# DRO-11 Implementation Plan

> **For agentic workers:** Use subagent-driven-development for the terrain task and independent review, with combat integration and native validation coordinated in this session.

**Goal:** Make a longer safe route and a timed electrical shortcut meaningful in the existing three-minute arena.

**Architecture:** Static world geometry provides shared solid/visibility queries, conservative swept actor collision, and a tiny authored waypoint graph. Flight still integrates thrust and momentum. Combat owns hazard damage and weapon interactions; a separate world scene renders geometry and hazard state.

**Tech Stack:** Rust, Bevy 0.19.1; no new dependencies.

## Global constraints

- Mix low cover with a tall divider, and keep the shortcut hazardous at every altitude.
- Preserve existing flight controls, charging rates, module costs, and encounter duration.
- Both routes remain traversable using ordinary flight with an empty battery.
- No general navigation mesh, physics-engine migration, procedural maps, moving/destructible obstacles, additional hazard types, new modules, pickups, campaign work, or map editor.
- Run `cargo fmt --check`, `cargo test`, and `cargo clippy --all-targets`; use `cargo dev` for native validation. Shared target cache: `/Users/shooshte/projects/drone-survivors/target`.

## Task 1: Terrain, physical movement, authored routes, spawn clearance

Files: create `src/world.rs`, `src/world/geometry.rs`, `src/world/navigation.rs`, and focused tests; modify `src/arena.rs`, `src/arena/flight.rs`, `src/combat/enemies.rs`, `src/combat/waves.rs`; declare `mod world` in `src/main.rs`.

Interfaces for other tasks:

```rust
pub(crate) struct Solid { pub center: Vec3, pub half: Vec3 }
pub(crate) struct WorldGeometry { pub solids: Vec<Solid>, pub hazard: Option<Solid> }
// Default is the authored playable map. No resource means unobstructed legacy fixture.
impl Solid {
    pub(crate) fn overlaps(&self, center: Vec3, half: Vec3) -> bool;
    pub(crate) fn segment_hit(&self, start: Vec3, end: Vec3, half: Vec3) -> Option<f32>;
}
impl WorldGeometry {
    pub(crate) fn first_hit(&self, start: Vec3, end: Vec3, radius: f32) -> Option<f32>;
    pub(crate) fn line_clear(&self, start: Vec3, end: Vec3) -> bool;
}
#[derive(Resource, Default)]
pub(crate) struct PlayerPath { pub segments: Vec<MotionSegment> }
pub(crate) struct MotionSegment {
    pub start: Vec3, pub end: Vec3, pub from: f32, pub to: f32, pub half: Vec3,
}
```

- [x] Write collision fixtures first: boosted 525-unit/s crossing, corner sliding, rotation beside walls, low-cover overflight, and floor-to-ceiling containment at 30/60/144 FPS plus a 100 ms hitch. Run the focused tests and record the missing-behavior failure.
- [x] Implement conservative swept collision within existing bounded flight steps, preserving tangential velocity. Record player and enemy movement segments for hazard and projectile consumers.
- [x] Author a small map with the existing center spawn and charger cylinders clear, one full-height divider with an electrical passage, a longer hazard-free detour, and low cover. Keep all passages comfortably wider than the rotated scout.
- [x] Add clearance-checked waypoint routing that sends the existing enemy pilot toward a reachable next point, with stable progress around corners. Test arrival from both sides, changes of target side, and normal-cap pursuit without five-second wall stalls.
- [x] Reject obstacle/hazard/disconnected spawn candidates both at warning creation and activation. Preserve existing clearance and cap semantics.
- [x] Run focused and legacy tests, review, and commit the terrain milestone.

## Task 2: Weapon occlusion and electrical hazard

Files: modify `src/combat/weapon.rs`, `src/combat/rockets.rs`, `src/combat.rs`, `src/combat/lifecycle.rs`; create `src/combat/hazards.rs`, `src/combat/world_tests.rs`; create hazard state in `src/world/hazard.rs`.

- [x] Write failing integration tests: obstructed-nearest target selection; earliest wall vs enemy hit; terrain rocket explosion with covered enemies excluded; low-cover firing above its top.
- [x] Share the world ray queries between target selection and impact. Compare impacts in the same normalized time units, let terrain win ties, and offset the explosion visibility origin onto the incoming side of a surface.
- [x] Write hazard tests for harmless warning, active crossing, one event per actor/window, shield/contact protection, enemy kills, depletion, restart, terminal freeze, and hitches.
- [x] Implement a 3-second inactive / 1-second warning / 1-second active cycle, 10 damage, and per-window successful-hit tracking. Intersect movement segments only with active time intervals; a long hitch may end a phase but must not skip the displayed warning or accumulate cycles of damage.
- [x] Stage module power before hazard/contact resolution. Reuse one player-damage helper so both sources share shield/protection/death handling. Hazard deaths use ordinary outcomes and kill accounting.
- [x] Run focused and complete headless tests, review, and commit the combat milestone.

## Task 3: Presentation and playable validation

Files: create `src/world/scene.rs`; wire world resources and scene in `src/main.rs`; update `src/combat/validation.rs` only as required for terrain-aware validation; update `README.md` and `docs/playtests.md`.

- [x] Draw reusable solid meshes with clear low/tall silhouettes and translucent tall faces/opaque boundary edges so the drone remains visible. Draw hazard floor/top/vertical boundaries, emitters, and distinct inactive/warning/active patterns.
- [x] Add compact text state/countdown and route labels without covering the existing HUD at 640 × 480. Freeze animation with gameplay and reset without allocating duplicate entities/assets.
- [x] Add meaningful scene/lifecycle assertions and run `cargo dev` with normal play and validation scenarios. Inspect screenshots at normal and minimum size; exercise route directions and timing, record observed travel times and limitations honestly.
- [x] Run `cargo fmt --check`, `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings`; review the entire branch and fix important findings.
- [x] Commit validation/docs, push `codex/dro-11-obstacles`, and open a PR against `main` with evidence and any remaining manual-playtest limitations.

## Progress

- Baseline: 113 tests passed on `5a6e2d9` in the isolated worktree.

- Implementation and independent review complete: 143 tests pass, fmt and strict Clippy pass.
- Native route and combat runs recorded in `docs/playtests.md`. Final compact HUD recapture remains an unlocked-session follow-up; earlier geometry captures were inspected.
