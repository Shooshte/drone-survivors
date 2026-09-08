# DRO-5 playable shell and test arena

Source: https://linear.app/drone-survivors/issue/DRO-5/01-playable-shell-and-test-arena

Build the issue's independently playable native macOS shell with the existing Rust/Bevy setup. The arena is a centered 960 × 540 world-unit rectangle. A fixed orthographic camera keeps the entire arena visible when resizing. A contrasting placeholder drone starts at the origin, with a visible home marker, arena grid, boundary, and control legend.

WASD and arrow keys move at 240 world units per second. Normalize the combined directional input, so diagonals and duplicate bindings cannot increase speed; opposite directions cancel. Clamp the drone center with its visual half-size included, keeping the whole drone inside the arena. `R` restores the starting transform and takes precedence over movement that frame. Escape closes the window.

Use a small gameplay plugin for configuration, spawn, input, movement, and reset, with scene presentation in a separate module. Test the actual ECS update systems using a controlled clock and keyboard resource, without requiring a GPU or window. Cover direction mappings, diagonal speed, elapsed-time equivalence, edges/corners, opposite input, and repeated reset while moving.

This direct ECS approach is sufficient for a rectangular empty arena. A physics engine would add an unnecessary dependency; a state machine or mission framework belongs to later chunks. No combat, enemies, progression, or external assets are required.

Validate with cargo fmt --check, cargo test, cargo clippy --all-targets, and a native cargo dev control smoke check. Record actual results and next steps in docs/playtests.md.
