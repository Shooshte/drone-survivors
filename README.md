# Drone Survivors

A Rust/Bevy prototype, currently featuring a playable 3D drone combat arena.
Configured using the [Bevy setup guide](https://bevy.org/learn/quick-start/getting-started/setup/).

## Development

Install Rust through rustup and the Xcode command line tools on macOS. The
project's `rust-toolchain.toml` selects nightly Rust and installs rustfmt,
Clippy, rust-analyzer, and the WebAssembly target. Enable rust-analyzer in your
editor to use the installed language server.

```sh
cargo dev
```

The test arena opens immediately. Use **WASD or arrow keys** to move over the
ground plane, **Space** to ascend, **either Shift** to descend, **R** to return
restart the encounter at the center and starting altitude, and **Escape** to quit. Release movement
keys to hover. Diagonal movement, including combined horizontal and vertical
flight, has the same total speed as straight movement.

The full drone stops at the ground, ceiling, and side walls; it can still move
along a boundary or back away from it. The 960 × 540 arena has a 300-unit
ceiling, and the drone starts 90 units above ground (measured at its center).
An angled camera keeps the full flight volume visible as the window is resized.
A ring on the ground and a vertical guide show the drone's ground position.

Three orange flying chasers pursue the drone at every altitude. The basic
weapon automatically fires yellow projectiles at the nearest enemy within
400 world units, aiming at its current position. Shots travel straight and
can miss; each disappears after its first hit, after one second, or on leaving
the arena. Keep moving to avoid contact.

The HUD shows hull health and remaining enemies. Contact deals 25 damage,
followed by a shared 0.75-second invulnerability window. At zero health, gameplay
freezes and **R** starts a fresh encounter. R also restarts during combat or
after clearing the arena, restoring health, enemy positions, and weapon timing.
There are no waves or respawns between restarts. Balance values are provisional
and grouped in `CombatConfig` in `src/combat.rs` for playtesting.

See [the control smoke check and playtest notes](docs/playtests.md).

This alias enables Bevy's dynamic linking for faster iterative builds. Dev
builds optimize project code at level 1 and dependencies at level 3. Native
macOS builds also enable nightly generic sharing and use Apple's default
linker. The first build after changing compiler or optimization settings
rebuilds the engine and can take several minutes.

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets
```

## Release builds

```sh
cargo build --release
```

Release builds use one codegen unit and thin link-time optimization. Dynamic
linking is enabled only by `cargo dev` or an explicit feature flag, so the
release command does not require shipping `libbevy_dylib`. For the `log` crate,
trace logging is compiled out in development; release builds retain only
warning and error logging.

## WebAssembly

```sh
cargo build --target wasm32-unknown-unknown --profile wasm-release
```

The `wasm-release` profile inherits the release optimizations, optimizes for
size, and strips debug information. This produces a raw Wasm artifact; a
playable browser app will also need web packaging when the game is implemented.

For an additional size optimization pass, install Binaryen (`brew install
binaryen` on macOS), then run:

```sh
wasm-opt -Os \
  target/wasm32-unknown-unknown/wasm-release/drone-survivors.wasm \
  --output target/wasm32-unknown-unknown/wasm-release/drone-survivors.optimized.wasm
```

When adding web packaging, apply this pass to the final packaged Wasm file.

## Platform choices

The Bevy guide recommends the default Apple linker on macOS; LLD and Mold are
alternatives for other platforms, not additional optimizations to stack here.
Cranelift is intentionally disabled because the guide reports crashes with
Bevy on macOS and no Wasm support. LLVM remains the code generation backend.
