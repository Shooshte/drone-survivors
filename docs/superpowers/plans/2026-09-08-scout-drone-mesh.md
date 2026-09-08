# Scout Drone Mesh Implementation Plan

**Goal:** Add a reference-inspired scout mesh to the combat test arena in codex/scout-drone-mesh.

**Architecture:** Author and export a Blender model using tools/build_scout_drone.py. Load its GLB scene beneath the existing Drone entity. Keep presentation independent of combat and movement.

**Tech Stack:** Rust, Bevy 0.19.1, locally installed Blender, glTF 2.0.

## Constraints

- Bounds: half extents (18, 6, 18), +Y up, -Z forward.
- Self-contained materials, no new Rust dependencies.
- Exactly two underside hover assemblies, four forward sensors and two subtle module housings.
- Preserve arena controls, collision, health, targeting and restart behavior.

## Tasks

- [x] Create isolated worktree and run baseline cargo test (31 passed).
- [x] Replace primitive-specific geometry test with GLB vertex/bounds validation; run it and observe missing asset failure.
- [x] Create tools/build_scout_drone.py, art/scout/scout_drone.blend and assets/models/scout_drone.glb. Bake modifiers, consolidate meshes by material, and render docs/images/scout-drone.png.
- [x] Replace placeholder construction in src/arena/scene.rs with a GLB WorldAssetRoot child, loaded once at startup.
- [x] Verify the actual asset loads through Bevy and survives restarts; keep existing movement/combat tests.
- [x] Run cargo fmt --check, cargo test and cargo clippy --all-targets -- -D warnings, inspect the arena and close-up render, and document rebuilding/playtesting in README.md and docs/playtests.md.

## Completed validation

- `cargo test --locked`: 32 passed.
- `cargo clippy --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --check` and `git diff --check`: passed.
- Blender studio render and native Bevy arena/close-up screenshots inspected.
- Independent read-only review: no actionable defects.
- Temporary screenshot harness removed. Workspace and branch retained for review.
