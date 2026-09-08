# Playable Shell Implementation Plan

> Execute inline, following the existing issue scope and test-driven development workflow.

**Goal:** Complete DRO-5 with a runnable bounded arena, predictable keyboard movement, and repeatable reset.

**Architecture:** `src/main.rs` configures the window and registers `ArenaPlugin`. `src/arena.rs` owns the gameplay systems and headless ECS tests; `src/arena/scene.rs` owns the camera, arena, placeholder drone visuals, and legend.

**Tech Stack:** Existing Rust nightly and Bevy 0.19.1; no new dependencies.

## Global constraints

Native macOS acceptance; preserve Cargo setup; use placeholder shapes; test meaningful behavior; document results and next steps in `docs/playtests.md`.

## Tasks

- [ ] Verify the untouched baseline with `cargo test --locked`.
- [ ] Add headless tests that drive keyboard input and a controlled `Time` resource through the actual gameplay plugin. Verify failures for missing movement, boundary clamp, and reset before implementing those behaviors.
- [ ] Implement the arena resource, drone component, and a single update system: map WASD/arrows, normalize direction, multiply speed by `Time::delta_secs`, clamp using drone half-size, and prioritize R reset.
- [ ] Add a camera with `ScalingMode::AutoMin`, floor/grid/boundary sprites, a drone with four rotor placeholders, home marker, and a text control legend. Configure a 1120 × 720 window in the entry point.
- [ ] Run the headless tests, formatting, and Clippy. Launch `cargo dev` and inspect the native window, controls, reset, and resize behavior.
- [ ] Update README controls and `docs/playtests.md` with evidence and next steps; review the complete diff.
- [ ] Commit coherent implementation, push `codex/dro-5-playable-shell`, and create a PR targeting `main`.
