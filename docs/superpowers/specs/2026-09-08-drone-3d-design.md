# 3D drone movement and bounded flight

Port the playable arena and drone to native Bevy 3D meshes. Keep gameplay in
`ArenaPlugin` and presentation in `ArenaScenePlugin`; no physics dependency is
needed for an empty rectangular flight volume.

Use X/Z for the 960 × 540 horizontal arena and Y for altitude. The ground is
Y = 0 and the ceiling is Y = 300. The drone starts at (0, 90, 0), moves at
240 world units per second, and hovers when no input is held. Its complete
visible geometry fits within half-extents (18, 6, 18).

WASD and arrows retain horizontal movement: W/Up moves toward negative Z,
S/Down toward positive Z, A/Left toward negative X, and D/Right toward positive
X. Space ascends and either Shift descends. Normalize the combined three-axis
input so horizontal, vertical, and diagonal flight have the same total speed.
Duplicate bindings do not increase speed and opposing bindings cancel.

Clamp each position axis independently using the drone half-extents. The drone
bottom stops at the ground and its top stops at the ceiling, even after a long
frame. Contact blocks only travel into that surface; movement along the surface
and away from it remains possible. R restores the starting transform, including
altitude, and takes precedence over movement on that frame. Escape quits.

Render a mesh body with four rotors, an X/Z ground grid, a center marker, and an
open frame outlining the flight volume. Use an angled orthographic Camera3d
with X aligned to screen horizontal and enough resize-safe framing to show the
entire volume. Lighting and a ground-position marker make altitude readable.
The on-screen legend documents the new controls.

Alternatives considered: simulated height on the current 2D sprites would not
provide a true 3D port; a physics engine is unnecessary for axis-aligned limits
without gravity or obstacles.

Validate the actual ECS movement system with controlled time and keyboard input:
all directions and aliases, two/three-axis normalization, opposing/released
input, frame-rate independence, all 26 boundary directions, sustained ground
and ceiling contact, sliding and leaving a boundary, and reset behavior.
Check formatting, tests, Clippy, and the native scene. Update README and playtest
instructions, then commit, push, and open a pull request against main.
