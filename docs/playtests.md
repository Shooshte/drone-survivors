# Playtests

## DRO-5 — Playable shell and test arena — 2026-09-08

Target: native macOS, keyboard input, existing Rust nightly / Bevy 0.19.1 setup.

### Repeatable control smoke check

1. Run `cargo dev`. Confirm a titled window opens with a visible arena, grid,
   center marker, placeholder drone, and control legend.
2. Move with W, A, S, D, then each arrow key. Release the keys; the drone stops.
   Try all four diagonal combinations. Their travel speed should match a
   straight direction. Combining W and Up should not add speed or skew a diagonal.
3. Hold movement toward each edge and each corner. All four rotors must remain
   inside the boundary. Move back inward to confirm the drone is not stuck.
4. Move away, press R, and confirm the drone returns exactly to the center marker.
   Repeat at least three times. Try R while holding a movement key: reset wins
   that frame, and held movement resumes on the next frame. Holding R should not
   repeatedly reset; release and press it to reset again.
5. Resize the window wider and taller. The complete arena remains visible and
   the drone's world-space bounds do not change.
6. Press Escape and confirm the application closes cleanly. Relaunch to confirm
   the same starting scene.

### Automated coverage

Seven tests drive the real gameplay plugin with keyboard input and a controlled
clock, without a display or GPU. They cover both keyboard layouts, eight movement
directions, normalized diagonals, duplicate and opposing bindings, stopping on
release, equal travel over one second at 30 and 120 updates, every edge/corner
including a ten-second update, repeated reset without entity duplication, and
reset precedence/held-key behavior.

The test-first check produced six expected failures against the stationary shell;
movement then passed five tests with two reset failures; adding reset passed all
seven tests.

### Results

- `cargo fmt --check`: passed.
- `cargo test --locked`: all seven tests passed.
- `cargo clippy --all-targets --locked -- -D warnings`: passed with no warnings.
- `cargo dev`: compiled and ran on macOS 26.6.2 / Apple Silicon without a
  reported runtime error. The UI tool cannot target a raw Cargo executable, so
  native interaction used a temporary app bundle containing that same binary
  with only its development-library lookup path adjusted; no bundle files are
  part of the repository.
- Native visual check: camera, full arena boundary, grid, center marker, drone,
  and control legend rendered correctly. Brief D input moved the drone and R
  returned it to the marker. Resizing from 1120 × 720 to approximately 700 × 480
  kept the whole arena and controls visible. Escape terminated the process;
  relaunch restored the initial scene.
- The UI tool sends very short taps, so sustained movement, all keyboard
  directions, frame-rate equivalence, every edge/corner, and repeated reset are
  verified by the automated ECS tests above. A human held-key feel check remains
  useful before tuning movement speed.
- Independent source review found no actionable correctness issues.

No known blocking gameplay defect was found. Speed (240 world units/second) and
arena dimensions (960 × 540) are initial playtest values.

### Next step

DRO-6: add the first chasing enemy, basic automatic weapon, damage, death, and
restart lifecycle. This arena currently has no combat or transient run entities;
reset restores the existing drone instead of recreating the scene.
