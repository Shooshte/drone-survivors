# DRO-18: Campaign save and resume

## Scope and decisions

DRO-14 through DRO-17 are complete on main. The ticket authorizes implementation;
no product ambiguity blocks this bounded native macOS feature.

Use one versioned JSON slot in the user's application data directory. Compared
with serializing the ECS world, an explicit campaign snapshot keeps temporary
combat data out and permits strict validation. Compared with multiple slots or
a database, one atomic file fits this prototype and keeps recovery inspectable.

Persist banked balances, passive ranks, module ownership/loadout, mission wins,
selected mission, settled attempt history and the next attempt counter. Restore
into the hub with no active attempt/result and pristine temporary state. Saving
an already credited result stores balance and completion together; loading never
calls settlement. Interrupted missions restore the last between-mission state.

## Storage and failure handling

Decode a versioned snapshot and reject malformed fields, unknown versions,
impossible prerequisites, ranks, ownership/loadouts, mission IDs and attempt IDs.
Save after permanent changes and finalization. Use a same-directory synced temp
file, an exclusive file lock, an expected-content check and atomic rename.
Keep a previous valid snapshot; confirmed replacement archives original bytes
under a unique name. Never overwrite invalid/incompatible input automatically.

Startup offers Continue when valid, New Campaign when missing, and retry or
confirmed recovery when unreadable/invalid/incompatible. New Campaign requires
confirmation for an existing slot. A failed autosave pauses progression behind a
retry screen and retains the in-memory transaction; retry cannot pay it twice.
Quitting with that error leaves the last successfully saved state on disk.

## Presentation and isolation

Reuse the existing cyan/blue menu design, keyboard and mouse controls and fresh
input gating. Show a clear saved/error status. Add a campaign-menu shortcut from
the hub. Standard validation fixtures install no persistence. An explicit save
path override supports isolated native checks and portable testing.

## Verification

Storage tests use temporary directories for round trips, malformed/future data,
missing files, conflict detection, backup retention and failure preservation.
ECS tests cover purchases, selection, result settlement/reload/replay, interrupted
missions, cancellation, fresh input and retry. Run fmt, all tests, Clippy and
native cargo dev checks at normal/minimum window sizes. Record evidence in
`docs/playtests.md`, commit in coherent chunks, push and open a PR against main.
