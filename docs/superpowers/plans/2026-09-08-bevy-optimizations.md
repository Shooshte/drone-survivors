# Bevy setup optimizations

Goal: apply the compatible optional optimizations from the Bevy setup guide to
this macOS project, including its previously missing dev profiles.

The user authorized these setup changes. Implement them directly in this folder.
Keep application behavior at the setup guide's Hello World stage.

- [x] Add dev, release, and wasm-release profiles and compile-time log filtering
  to `Cargo.toml`.
- [x] Add `cargo dev` for dynamic linking in `.cargo/config.toml`; leave release
  feature defaults static. Enable generic sharing for both native macOS targets.
- [x] Select nightly and install editor/checking components and the Wasm target
  through `rust-toolchain.toml`.
- [x] Document development, release, Wasm, and Binaryen commands in `README.md`.
  Retain Apple's linker and LLVM because the guide recommends them on macOS.
- [x] Install the selected toolchain and Binaryen; refresh `Cargo.lock`.
- [x] Run `cargo dev --locked` and require successful Hello World output.
- [x] Run `cargo check --locked --release` and
  `cargo check --locked --target wasm32-unknown-unknown --profile wasm-release`.
- [x] Run `cargo fmt --check`, inspect resolved features to confirm dynamic
  linking is absent by default, and review final configuration and lockfile.

Configuration changes are verified through Cargo itself; no application
behavior or test-only source code is needed.

Verification completed on 2026-09-08 with Rust 1.100.0-nightly and Binaryen 132.
The development run printed `Hello, world!`; native release and WebAssembly
release checks passed. Cargo reports the expected unused Bevy dependency while
the executable is still the setup guide's Hello World starter. No full release
binary or packaged browser build was produced during these checks.
