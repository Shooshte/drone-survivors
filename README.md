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

The test arena opens immediately. Fly relative to the drone's heading:

| Keys | Control |
| --- | --- |
| **W/S** or **Up/Down** | Pitch forward/backward |
| **A/D** or **Left/Right** | Turn left/right |
| **Q/E** | Bank left/right |
| **Space** / **either Shift** | Boost/reduce rotor thrust |
| **1–4** | Toggle the corresponding equipped module |
| **R** | Restart the encounter at the center, level and stationary |
| **Escape** | Quit |

Pitching and banking redirect rotor thrust to accelerate the drone horizontally.
Both reduce upward lift, so expect to lose altitude unless you add thrust.
Release pitch/bank controls to smoothly level out; momentum remains and drag
gradually slows the drift. Tilt in the opposite direction to brake. Turning
changes where the nose points and where tilted thrust pushes, while existing
momentum keeps its world direction.

Space boosts thrust and Shift reduces it. Releasing both restores the thrust
needed to hover **when level**; it does not immediately stop a climb or descent,
or recover lost altitude. Opposing keys cancel on each control axis and duplicate
bindings add no extra input. Pitch and bank share a 30-degree total tilt limit,
and all directions share a maximum horizontal speed of 420 world units/second.
Vertical motion is independent of that speed limit. Flight tuning values are
grouped in `FlightConfig` in `src/arena/flight.rs`.

The scout reaches full tilt in 0.125 seconds, turns at up to 240 degrees/second,
and has three times the original horizontal thrust acceleration. Space/Shift
retain their original level-flight vertical acceleration; pitching or banking
still costs altitude.

The full rotated drone stays inside the ground, ceiling, and side walls. Contact
removes velocity into the surface while preserving motion along or away from it.
The 960 × 540 arena has a 300-unit
ceiling, and the drone starts 90 units above ground (measured at its center).
An angled camera keeps the full flight volume visible as the window is resized.
A ring on the ground and a vertical guide show the drone's ground position.

Orange flying chasers arrive in timed bursts and pursue the drone at every altitude using the same
thrust, gravity, and momentum model. They start at rest, turn and bank to steer,
and brake as they approach. Sharp changes of direction require them to redirect
their momentum. Their 260-unit/second horizontal speed cap and slower tilt/turn
response give the scout an advantage in speed and agility. The basic
weapon automatically fires yellow projectiles at the nearest enemy within
400 world units, aiming at its current position. Shots travel straight and
can miss; each disappears after its first hit, after one second, or on leaving
the arena. Keep moving to avoid contact.

The five-minute encounter starts with three quiet seconds, then five enemies
every eight seconds.
Pressure increases through larger bursts and shorter intervals:

| Active time | Enemies per burst | Interval |
| --- | --- | --- |
| 0–30 seconds | 5 | 8 seconds; first warning at 3 seconds |
| 30–105 seconds | 7 | 8 seconds |
| 105–165 seconds | 8 | 6 seconds |
| 165–225 seconds | 9 | 5 seconds |
| 225–300 seconds | 11 | 4 seconds |

Each stage ends with at least six seconds without new spawn warnings. The
schedule requests 447 enemies in 50 bursts before spawn rejection or skipped
bursts. Surviving is intended to take a few attempts; that difficulty target
still needs human playtesting.

Only enemy numbers increase. Chaser health, damage, movement, and behavior stay
constant. Pink rings warn of incoming enemies for 0.75 seconds. Spawns are
cancelled if their position becomes unsafe; living enemies and pending warnings
share a cap of 30. Rejected spawns are discarded. Lulls leave surviving enemies
in play, and existing warnings may still finish. Nearby chasers steer apart
while retaining their physical flight.

The HUD shows hull, time remaining, living enemies, kills, and the current phase
or spawning lull. Chasers take two 10-damage hits to kill. Hits briefly flash the
affected enemy; a small amber burst marks a kill. Contact deals 10 hull damage,
followed by 0.75 seconds of shared invulnerability. The HUD flashes red on damage
and shows cyan **HULL PROTECTED** during that protection window.

At zero hull, gameplay freezes and **R** restarts. Survive until 5:00 to freeze
the encounter with **SURVIVED** and your kill count; remaining enemies do not need
to be cleared. R also restarts during combat or lulls, clearing enemies, shots,
warnings, effects, kills, timers, and flight momentum. Balance values are
provisional and grouped in `CombatConfig`, `WaveConfig`, and `FeedbackConfig`.

### Energy and charging

Four interchangeable module slots start equipped and switched off:

| Key | Module | Powered benefit | Drain |
| --- | --- | --- | --- |
| **1** | Weapon overdrive | Double basic firing rate | 10 energy/s |
| **2** | Shield | Block one direct hit; recharge in 5 powered seconds | 8 energy/s |
| **3** | Mobility | +25% horizontal acceleration and speed cap | 8 energy/s |
| **4** | Rocket launcher | Automatic rocket every 2 seconds, 20 splash damage within 70 units | 10 energy/s |

The battery starts full at 100. Each module requires at least 10 stored energy
to switch on, with no activation fee. Enabled modules drain continuously,
including when there is no target, while stationary, or while the shield is
ready/recharging. Holding a number key does not repeatedly toggle it.

The shield starts with one ready block. Blocking prevents the whole direct hit
and grants the ordinary 0.75-second contact protection window. Its recharge
advances only while powered; switching off or depletion pauses progress.
Toggling never restores a block. Mobility changes horizontal thrust acceleration
and the speed limit from 420 to 525, preserving vertical flight and handling.
Its state is applied on the next movement update, including restoring the normal
speed cap when disabled. Enemy flight is unchanged.

Blue rockets aim automatically at the nearest enemy within 400 units. They fly
straight at 500 units/second and explode on the first enemy hit, damaging each
enemy within the 70-unit 3D impact radius once, including the struck enemy.
There is no player splash damage. Misses expire after 1.5 seconds or leave the
arena. Switching off prevents new launches; existing rockets finish normally.
Rocket cooldown continues while off, so toggling cannot grant extra shots.
Weapon overdrive affects only basic fire.

At zero energy all modules switch off; basic automatic fire and every normal
flight control remain available. Charging never reactivates modules automatically.
Press the corresponding key when enough energy is available. Death/survival
freezes power; R restores full energy, all modules off, a ready shield, and clears
rockets, cooldowns and feedback.

Two cyan charging fields sit on opposite sides of the arena. Enter a field
with the drone's center below its visible top ring (height 160) to gain
25 energy/second. Fly and fight freely while charging; leaving stops recharge.
Fields have radius 90, never run out, and provide no protection. Overlapping
fields do not stack. All four modules drain 36/s, so even charging produces a
net loss of 11/s. Rockets alone leave a net gain of 15/s at a charger.

The HUD shows each slot's key, state and current/configured drain, shield
readiness/recharge progress, shared battery, total drain and signed net charging.
Failed activation identifies the affected slot. Any module can occupy any slot,
and empty slots are safe; a type cannot be duplicated. This prototype uses a fixed
loadout; purchasing and player-controlled rearrangement belong to DRO-16.
Numeric defaults live in `ModuleConfig` in `src/modules.rs` and `EnergyConfig`
in `src/energy.rs` and remain provisional playtest values.

### Obstacles, hazards, and routes

Low blocks provide cover and can be flown over. The tall divider reaches the
ceiling: use its central electrical passage or follow the green dotted detour.
Gold dots mark the shorter crossing between chargers. The detour avoids the
field, but enemies can follow either route.

The full-height electrical field repeats **3 seconds open → 1 second warning →
1 second active**. Warning beams flash before becoming continuous during damage.
Each active window can deal 10 damage once to each exposed drone, including
chasers. A ready powered shield blocks one event; hazard and enemy contact share
the usual protection window. Waiting, mobility, shielding, or luring pursuers
through the field can change which route is useful. Neither route requires power.
R resets the cycle; upgrade choices, death, and survival freeze it.

Solid terrain stops the entire rotated drone and enemy bodies while allowing
sliding. Both weapons target visible enemies; bullets stop at cover, rockets
explode on it, and solid cover blocks splash damage. Enemy pilots use a small
authored route graph while retaining their normal thrust and momentum.

### Experience and temporary choices

Kills award 4 XP throughout the encounter to offset the denser opening.
One green exploration pickup at the far side
of the arena grants 30 XP when the drone's center enters its visible 30-unit
sphere; it can be collected once per run. The HUD shows level, XP to the next
level, and acquired upgrades. Once no eligible upgrades remain, it immediately
shows **Build complete** and the acquired upgrades instead of level/XP progress.
XP accounting continues internally, but no further choices are promised.
Start at level 1: the first choice costs 50 XP,
then each level costs 25 more. Excess XP carries over.

Leveling pauses the encounter and offers up to three eligible upgrades. Select
with a fresh **1–3** press or click a card. **Backspace** or **Skip this upgrade**
declines the offer: the level remains earned, no effect is applied, and the
choice is spent. Skipped cards can appear later. If several levels are earned
together, resolve them one at a time; release the selection key/mouse button
between offers. Empty pools resume automatically. The prototype uses a fixed
offer seed for repeatable playtests.

| Upgrade | Benefit | Drawback |
| --- | --- | --- |
| Interceptor | +30% horizontal acceleration and speed cap | −25% battery capacity |
| Agile frame | +40% turn, tilt and leveling response | −20 maximum hull |
| Heavy armor | +50 maximum hull | −25% acceleration in all movement directions |
| Heavy rounds | Double basic projectile damage | −10% acceleration in all movement directions |
| Rapid shield | Powered recharge 5s → 2.5s | Shield drain 8 → 12 energy/s |
| Wide-area rockets | Explosion radius 70 → 105 | Launch interval 2s → 3s |

Each upgrade can be taken once. Shield/rocket choices require the corresponding
equipped module. Acceleration penalties include climbing, descending and braking;
neutral hover, angular handling and speed caps are preserved unless another
selected upgrade changes them. Percentage modifiers multiply, while hull changes
add: both armor and heavy rounds give 67.5% baseline acceleration; armor plus
agile frame gives 130 maximum hull. Heavy rounds preserves firing rate and free
basic fire at an empty battery.

Additional maximum hull adds the same amount to current hull. Reduced maximum
hull or battery capacity clamps the current value. Module power state, shield
blocks, and remaining cooldown/recharge fractions survive a choice. Already
launched shots keep their original damage and explosion radius.

![Temporary upgrade choices at 640×480](docs/images/experience-choices.png)

The selection screen freezes all gameplay time, including energy, charging,
protection, spawning and projectiles. Selecting cannot toggle a module or consume
the next queued choice with held input. **R** restarts even during a choice,
restoring baseline stats, XP, pickup, modules and the encounter. Death or victory
on a threshold frame takes precedence over selection. All tuning is provisional.

### Charger-camping balance comparison

```sh
cargo test --locked camping_balance_probe -- --ignored --nocapture
```

This opt-in 30 Hz simulation compares the previous and current waves/XP at both
chargers, with overdrive and overdrive-plus-shield builds, plus a moving keyboard
pilot. It uses the actual terrain/hazard and normally earned upgrade choices.
Camping fixtures start at a charger; the harness does not hold their position,
refill health, or grant XP. Output includes outcomes, choice times, energy,
peak population and spawn accounting. It is not a rendering benchmark or a human
playtest; surviving camping probes leave the experiential gate pending.

### Repeatable native validation

```sh
cargo dev -- --validate manual --seconds 600
cargo dev -- --validate survival
cargo dev -- --validate idle
cargo dev -- --validate routes --seconds 40
cargo dev -- --validate mobile
cargo dev -- --validate armored
cargo dev -- --validate choices
cargo dev -- --validate stress --enemies 150 --seconds 30
```

Manual records a human-controlled run without driving the drone, choosing
upgrades, granting XP, or changing gameplay. It logs module changes, charging,
route crossings, upgrade resolutions, and spawn pressure. Use `--seconds 600`
to leave time for reading upgrade choices; the limit includes wall time. R
restarts and clears the observations for the new attempt. Completed results print
immediately on death or survival, before the two-second exit delay. Save terminal output
alongside the human observations in [the playtest checklist](docs/playtests/dro-12-checklist.md).

Survival uses a repeatable keyboard pilot with collision avoidance; it does not
change health, damage, movement physics, or the authored waves. Idle applies no
pilot input and exercises stationary combat/death. These runs print progress and
exit two seconds after an outcome, or after the sampling limit plus warm-up.
R and Escape remain available. A human handling/readability playtest is still
needed alongside these repeatable checks.

Routes is a separate fixture with authored waves disabled. It uses ordinary
keyboard flight with every module off to traverse the shortcut in both directions,
then the detour in both directions. It waits for an open window and reports each
trip's travel and waiting time. It does not override hull or move the drone by
writing its position. The corresponding tests repeat at 30/60/144 FPS with an
empty battery and recharge disabled in the test fixture.

To capture the three hazard states during native validation:

```sh
DRONE_CAPTURE_DIR=/tmp/drone-captures cargo dev -- --validate idle --seconds 20
DRONE_CAPTURE_DIR=/tmp/drone-captures DRONE_CAPTURE_MINIMUM=1 cargo dev -- --validate idle --seconds 20
```

The second command uses the minimum 640 × 480 logical window. These environment
options are ignored during ordinary play. Keep the macOS session unlocked for
usable window captures; locked-session captures may be black.
Mobile and armored use the same normal encounter and pilot, selecting only
naturally earned upgrades through ordinary choice controls. Mobile prefers
Interceptor, Agile frame, and Rapid shield; armored prefers Heavy armor,
Heavy rounds, and Wide-area rockets. Other offers are skipped. Each resolution
logs its time and chosen effect; the final report includes acquired upgrades
and battery capacity. Survival, idle, routes, and stress skip earned choices so those
existing scenarios continue.

Choices is an explicit UI preview: it starts with 140 synthetic XP at 640×480,
leaves choice/Skip input to the player, and saves a native screenshot to
`/tmp/dro10-choices.png` after one real second of selection. It defaults to a
60-second sampling limit; `--seconds` can shorten it. Synthetic XP and the
preview window size are confined to this mode.

Stress is a separate workload: it disables authored waves, places 150 chasers in
a fixed grid, enables player invulnerability, and replaces kills to maintain the
requested population. Real pursuit, separation, projectiles, damage to enemies,
and feedback remain active. Stress spawning deliberately bypasses normal warnings
and clearance; it is not the playable encounter. Only stress accepts `--enemies`.

All validation modes report wall-clock median/p95/p99 frame times and hitches over
33.3 ms, actual enemy-count range, projectile peak, hit/kill/damage counts, and
physical resolution. The first five seconds and frozen terminal states are
excluded. `--seconds` bounds the post-warm-up run (1–600 seconds). Normal launch
installs no validation systems or gameplay overrides.

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

## Scout drone model

The combat player uses an original Blender scout: split silver armor over a
black chassis, three compact hover rotors (one under each wing and one under
the nose), cyan lights, a four-lens sensor
cluster, swept fins, and two small equipment housings. This is a visual asset;
XP gain, terrain immunity and core-slot passives are not implemented yet.

![Scout studio preview](docs/images/scout-drone.png)

- Game asset: `assets/models/scout_drone.glb` (self-contained, six material meshes).
- Editable source: `art/scout/scout_drone.blend` (named parts and studio setup).
- Rebuild script: `tools/build_scout_drone.py` (tested with Blender 5.2.1).

Run from this checkout so Bevy finds `assets/`. Include that directory alongside
any distributed executable. The model uses +Y up, -Z forward and fits the existing
70 × 30 × 90 collision envelope. The model is 2.5× its original size for arena
readability; rotor diameter is 38% smaller relative to the body. Restarting an
encounter retains the loaded model.

To regenerate the Blender source, GLB and studio preview (overwriting those files):

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
  --python tools/build_scout_drone.py
```

On other platforms, use your Blender executable in place of the macOS path.
The script recreates the model from its parameters; edits made directly in the
`.blend` must be exported separately to preserve them. Export only the drone mesh
objects as a GLB with +Y up, excluding the studio floor, camera and lights.

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
