# Mission 01 Blockout Implementation Plan

> **For agentic workers:** Use subagent-driven-development for bounded independent work and review; execute integration in this session. Steps use checkbox syntax for tracking.

**Goal:** Ship DRO-42 as a playable sketch-based payload mission, preserving other missions and save compatibility.
**Architecture:** Mission-local configuration supplies world geometry, landmarks and pacing. Existing flight/combat/economy remain authoritative; a run resource tracks payload, dormant encounters and the one-shot holdout. Reset selects/restores map resources and presentation.
**Tech Stack:** Rust, Bevy 0.19.1, existing native validation and test harnesses.

## Global Constraints

- All hatched terrain blocks flight at every altitude.
- Start southeast, collect one payload west, deliver northeast; about 180 seconds ordinary flight excluding combat/detours.
- Reserve one of four module slots before launch; three remain available for equipment.
- Delivery with payload immediately wins, including a lethal hit during entry; earlier lethal damage fails.
- Groups 4a/4b: ten chasers each; group 6: twenty; groups 9/10: two slowers and five chasers each.
- Permanent sources 8/11 start on pickup, persist indefinitely and cannot be destroyed. Bound population/backlog, preserve fixed counts and source fairness.
- Optional challenge: one attempt, three waves at 0/10/20 active seconds, completion at 30, no kill quota, early exit forfeits reward, pause freezes timer, fresh run restores availability.
- Hidden reward and challenge award components. Other missions/catalog and saved module ownership remain intact.
- Existing directional travel rules: +40% with, -40% against; perpendicular/vertical unchanged, no overlap stacking.
- Formatting, locked tests, strict Clippy, native 1120×720 and 640×480 checks; record agent evidence separately from human acceptance.

### Task 1: Authored geometry, navigation and directional fields
**Files:** new src/world/mission01.rs; src/world/navigation.rs, geometry.rs, environment.rs and focused tests.
**Interface:** mission01::point(east: f32, south: f32) -> Vec3; geometry() -> WorldGeometry; arena() -> Arena; start() -> Vec3; chargers() -> [(&'static str, Vec3); 4]. Supply named pickup/delivery/hidden/challenge landmarks and route waypoints. Mission geometry carries precomputed navigation, default geometry preserves existing behavior.
- [ ] Add failing route/body clearance and arbitrary-direction field tests; run targeted tests and observe missing behavior.
- [ ] Implement schematic solids and graph with bounded runtime work; flight at any altitude cannot cross terrain. Keep coordinates and scale centralized.
- [ ] Implement field projection: `d + direction * (d.dot(direction) * if d.dot(direction) >= 0. { 0.4 } else { -0.4 })`; choose one overlapping field deterministically.
- [ ] Measure actual flight route, validate all landmark approaches and graph pursuit; tune scale toward 180 seconds.
- [ ] Run targeted tests, review and commit geometry/field work.

### Task 2: Payload, equipment and map lifecycle
**Files:** src/mission/blockout.rs and focused tests; mission.rs/objectives.rs; modules UI/input; arena/energy/economy setup and scene refresh.
**Interface:** BlockoutRun resource identifies enabled mission and stores cargo/holdout state. Configuration runs in Baseline before resets; map presentation refreshes on resource changes.
- [ ] Add failing launch, no-timer-win, payload, ordered lethal delivery, empty-loadout, restart and map-switch tests.
- [ ] Add Mission 01 objective and slot-4 reservation. Refuse incompatible launch with actionable text, retain ownership and saved equipment until user edits.
- [ ] Select Mission 01 geometry, start, chargers and component reward; restore defaults for other missions. Reuse scene assets and replace stale map visuals.
- [ ] Resolve actual motion-segment pickup/delivery and contact in travel order; tie favors delivery.
- [ ] Verify normal mission lifecycle, progression/save replay, cleanup; commit.

### Task 3: Authored encounters and optional holdout
**Files:** new src/combat/mission01.rs and tests, combat scheduling; blockout run challenge state and HUD.
**Interface:** consume mission01 landmarks and BlockoutRun payload/holdout state, spawn existing enemy kinds through existing enemy constructor.
- [ ] Add failing proximity/count/cap/source-fairness/holdout timing tests.
- [ ] Activate each group once and preserve every authored request. Permanently enable both sources on pickup; bounded active cap and coalesced pending source requests with round-robin admission.
- [ ] Start challenge on first entry; emit three timed waves; exit consumes attempt; completion credits components once while enemies can remain. Reset clears all state.
- [ ] Ensure spawned enemies give normal XP, collision-safe spawn and pursuit, no shared global bursts; test and commit.

### Task 4: Readability, full validation and delivery
**Files:** map scene/minimap or guidance, docs/playtests.md, validation fixtures/screenshots.
- [ ] Show payload reservation, pickup/delivery, source and challenge state, terrain and directional arrows at both window sizes.
- [ ] Run route pilot without combat and with fields; record loadout, travel time and clearance. Run bounded pressure probe and native captures.
- [ ] Run `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings`.
- [ ] Review branch against issue; fix substantive findings and verify affected checks.
- [ ] Commit evidence, push `codex/dro-42-mission-blockout`, open PR against main and attach it to this task. Record outstanding human acceptance honestly.
