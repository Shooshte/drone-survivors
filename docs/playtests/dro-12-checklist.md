# DRO-12 human playtest checklist

Use the PR build on macOS. The goal is to test learning, readability, and whether
two different tactics can survive five active minutes. Automated runs do not
supply this evidence. Record failed attempts as well as wins.

## Setup

Run `cargo dev -- --validate manual --seconds 600` from the project and save the
terminal output. Manual mode leaves flight, module keys, mouse/keyboard upgrade
choices, and Skip to the player. It applies no combat overrides. The ten-minute
wall-time limit allows pauses to read cards; survival still requires five minutes
of gameplay. Results print immediately on death or survival, before the two-second
exit delay. R starts a fresh attempt and clears its measurements; keep the terminal
log to retain previous results. Escape quits. Launch again if the process exits.

Use two players unfamiliar with the game when available. Otherwise record/review
two fresh runs yourself and mark the same-player fallback. The fallback cannot
establish how quickly an unfamiliar player learns. Record screen video with your
preferred recorder if using the recorded-run fallback.

Before playing, read the README controls. Give unfamiliar players only those
controls; let them choose their own tactics before offering strategy suggestions.

## Record for each attempt

| Field | Observation |
| --- | --- |
| Build commit, Mac/OS, logical window size | |
| Player and prior familiarity | |
| Attempt number; result and active time | |
| Cause of death, or most dangerous moment | |
| First choice time; selections and skips | |
| Intended tactic and what actually changed | |
| A useful module/charging decision and why it helped | |
| Shortcut/detour choice and reason | |
| Upgrade benefit that felt stronger | |
| Upgrade drawback that changed a decision | |
| When pressure first became difficult; whether it kept increasing | |
| HUD, orientation, hazard or choice readability problem | |
| Recording/log filename | |

The provisional first-win target is roughly 2–5 attempts for a controls-familiar
newcomer. Report actual results; do not coach or omit failures to fit the target.

## Two successful tactics

Obtain two normal five-minute wins with differences in power or route use, not
just different upgrade names. Possible examples are mobile conservation and
heavier powered offense, but player-discovered tactics are equally valid.
For a self-play fallback, try those contrasts after the initial uncoached attempt.

At normal size and 640 × 480, check drone orientation, hull/energy, each module,
incoming warnings, hazard phase, and choice benefits/drawbacks under pressure.
Record if text is technically present but makes the arena hard to see.

Exercise R during combat, a warning, and a choice, plus death/restart. Confirm a
fresh timer, full baseline hull/battery, modules off, restored pickup, cleared
upgrades, and the initial hazard cycle. A long choice pause should advance none
of the enemies, shots, charging, or hazard warnings and cause no catch-up on resume.

## Gate decision

- [ ] Opening allows orientation; pressure accelerates after approximately 30 seconds.
- [ ] Final minute challenges an upgraded build instead of flattening at the cap.
- [ ] Two meaningfully different tactics each survive 300 active seconds.
- [ ] Players can explain power decisions and upgrade benefits/drawbacks.
- [ ] After the final eligible upgrade, Build complete replaces level/XP progress immediately; no empty choice appears.
- [ ] Critical information stays readable at both window sizes.
- [ ] Highest-impact observed problems were fixed and affected scenarios repeated.
- [ ] Evidence limitations, including same-player fallback, are recorded honestly.

Append observations and the explicit pass/pending decision to docs/playtests.md.
Keep DRO-12 pending if required evidence is missing; the PR's automated checks
establish implementation correctness, not that this experiential gate passed.
