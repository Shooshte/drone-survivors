# DRO-13 — One complete mission lifecycle

## Outcome and scope

Chunk 9 of 24 in the Drone Survivors development work plan.
**Phase B — Build the mission and progression loop**

**Depends on:** [Chunk 08: Combat prototype gate](<https://linear.app/drone-survivors/issue/DRO-12/08-combat-prototype-gate>)

## Approved implementation scope — September 13, 2026

Wrap the existing combat encounter in one complete, repeatable mission lifecycle. Reuse the current integrated arena, combat tuning, flight, charging, modules, and temporary upgrades.

### Hub, briefing, and launch

* Normal startup opens a minimal hub. The flow is Hub → Briefing → Launch → Combat → Results → Hub, with one freely replayable mission.
* Hub and briefing may contain placeholder narrative text. Functional labels, the survival objective, duration, and controls must remain accurate and understandable; polished story content is not required.
* The hub exposes a Mission briefing button; the briefing exposes Launch and Back to hub. Launch is an action, not an additional required screen.
* Lock drone and module selection to the current Scout and existing four-module loadout. Display the fixed loadout as read-only information; no drone picker, module swapping, purchasing, or loadout editor is required. Normal in-combat module toggles and temporary upgrade choices remain available.
* Use clickable buttons and keyboard equivalents: Enter opens the briefing from the hub, launches from the briefing, or returns to the hub from results; Backspace returns from briefing to hub. Each transition requires fresh input, so one held key or click cannot advance through multiple screens.
* Preserve Escape as quit throughout. R restarts a fresh attempt during combat or an upgrade choice, using the same clean-launch path. R has no action in hub, briefing, or results. An interrupted attempt produces no success/failure completion record; the replacement receives a new attempt identity.

### Objective and results

* Reuse the current 300 active-second survival objective. An alive player succeeds at the limit without clearing remaining enemies; zero hull fails the mission. Upgrade choices pause the objective timer.
* Preserve outcome precedence: restart first, fatal damage before simultaneous survival, and terminal outcomes before opening another upgrade choice.
* Freeze gameplay in hub, briefing, results, and upgrade selection, including flight, combat, hazards, spawning, energy/charger recovery, cooldowns, and protection timers. Resuming or launching must not catch up time spent on those screens.
* Results show success/failure, active elapsed time, and kills, with a Return to hub action. Capture those values before resetting mission state and keep the displayed result stable.
* Each launched or restarted attempt has its own identity. Finalize at most once, recording exactly one success/failure result for a terminal attempt. Repeated frames, duplicate input, or revisiting presentation must not duplicate completion or campaign records. Both success and failure permit returning to the hub and launching again.

### Mission state and campaign state

* Keep campaign data separate from the active mission. For this chunk, retain an in-memory history of completed attempt results and whether the single mission has ever succeeded; both survive hub returns and subsequent mission launches. Failure must not erase an earlier success.
* Campaign state lasts for the application session only. Disk saving, continue/new-campaign flows, and recovery after application restart belong to DRO-18.
* Every launch/restart restores the existing mission baseline: hull and protection; position, altitude hold, rotation and momentum; full battery; fixed modules switched off; ready shield; weapon/rocket cooldowns; full charger reserves and recovery state; hazard cycle; enemies, projectiles, warnings and effects; objective timer, kills, wave/spawn bookkeeping and deterministic seeds; XP, choice budget, pending choices, selected temporary upgrades, derived stats, and the exploration pickup.
* Exiting an attempt must leave no active mission entities or callbacks that can affect hub/briefing or a later attempt. Reusing static arena assets is acceptable; repeated cycles must not accumulate duplicate scene, camera, or UI entities.
* Reuse and coordinate the existing reset behavior behind explicit mission transitions rather than relying on a synthetic R keypress. Preserve the existing opt-in validation fixtures and their documented direct-combat behavior.

### Scope boundaries

Resource pickups, reward amounts and banking transactions belong to DRO-14. Permanent passives and editable equipment remain DRO-15/DRO-16. Mission selection and campaign unlocks belong to DRO-17; disk persistence to DRO-18; reconnaissance/extraction and reusable additional objectives to DRO-19. This chunk establishes a stable result and completion boundary for those later features without implementing them.

## Work checklist and acceptance checks

- [ ] Normal startup reaches the hub; placeholder hub/briefing content, fixed loadout display, launch, results, and return controls work by keyboard and mouse at 1120×720 and 640×480.
- [ ] Complete repeated Hub → Briefing → Combat → Success/Failure → Results → Hub cycles, including a success followed by failure and another launch. Both outcomes leave the mission replayable.
- [ ] The objective counts only active gameplay. Test a choice pause near the deadline and simultaneous fatal damage/survival, preserving the stated precedence.
- [ ] Completion and campaign recording occur once per terminal attempt under duplicate input and repeated updates. A restart starts a distinct clean attempt without completing the interrupted one.
- [ ] Launch and restart reset all mission state listed above, including from an active upgrade choice; campaign history and prior success survive subsequent attempts.
- [ ] No gameplay advances in hub, briefing, results, or choices, and no stale input, timers, entities, pending choice, or completion affects a new attempt. Repeated cycles do not accumulate scene/UI entities.
- [ ] Existing direct-combat validation modes remain usable. Add meaningful lifecycle/state-boundary tests, then demonstrate ordinary native launch, success/failure results, and repeat play.

## Shared context

Build a roughly 90-minute, 12-mission campaign where tactical decisions make the player feel clever and combat progression makes them feel powerful. Use Rust/Bevy and the existing Cargo setup. Initial target: native macOS with keyboard/mouse. Use reusable content and placeholders until the full loop is proven. Numeric balance values are playtest starting points.

## Definition of done

- [ ] Acceptance checks above pass and the deliverable can be demonstrated independently of unfinished downstream features.
- [ ] No known blocking defect in the changed behavior; record results and next steps in `docs/playtests.md`.
- [ ] For code changes, run `cargo fmt --check`, `cargo test`, and `cargo clippy --all-targets`; use `cargo dev` for the manual gameplay checks. Test meaningful rules and edge cases, not copies of static content.
- [ ] Review and commit this chunk separately.

Source: `docs/superpowers/plans/2026-09-08-game-development-work-plan.md` (September 8, 2026).
