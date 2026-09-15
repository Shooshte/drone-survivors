# DRO-19: Reusable mission objectives

User approved the scope on September 15, 2026; recorded in Linear before implementation.

Mission 02 is reconnaissance (visit three distinct scan sites, then extract).
Mission 03 is extraction (collect three dedicated cargo pickups, then extract).
Other missions retain the 300-active-second survival objective. Reuse the arena,
combat schedule, campaign unlock graph and reward/save boundaries. Cargo never
enters the resource wallet. Reaching extraction early does not finish a mission.
Proximity is automatic, in 3D, with terrain line-of-sight; sites count once.

Use a typed objective kind selected from the active mission at launch and a
small attempt-local progress resource. A configurable set of three reachable
positions and an extraction position supplies reusable fixtures and placeholder
content. Sites sit at normal flight height (90); a 70-unit visit radius and
95-unit extraction radius are provisional tuning. Sites and extraction are
separate, visible landmarks, with progress and nearest-target guidance in HUD.
Briefing and selection identify the objective and explain automatic proximity.

Progress resets on launch/restart and cannot advance in menus, upgrades or
results. Restart takes precedence; fatal damage beats extraction, which beats
opening another upgrade choice. Completion still uses the existing exactly-once
result/reward/save transaction. No mid-mission persistence or save migration.
Elapsed time continues beyond 300 for the new types, without a timer victory or
timeout; reuse the existing finite wave schedule (no new bursts after its end,
remaining enemies persist). Results use objective-neutral success wording.

Verification: tiny ECS fixtures exercise incomplete and repeated triggers,
early extraction, fatal damage, success, pauses/restart, reward settlement and
save reload. Check reachable placements against arena geometry. Update existing
synthetic campaign fixtures to finish via objective triggers. Run fmt, test,
strict Clippy, native cargo dev fixtures at 1120×720 and 640×480, plus physical
navigation. Record evidence and review before pushing and opening a PR to main.
