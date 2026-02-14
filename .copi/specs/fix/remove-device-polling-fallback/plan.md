# Spec: Remove polling fallback for device events

- Branch: `fix/remove-device-polling-fallback`
- Source: User brief (2026-01-24) — “remove old/backup polling for device events logic”
- Related work: PR #23 (signal-based device change detection)

## Context

Device insert/remove detection is now primarily signal-based via UDisks2’s `org.freedesktop.DBus.ObjectManager` signals. The codebase still contains a polling-based fallback stream that periodically calls `org.freedesktop.UDisks2.Manager.GetBlockDevices()` and diffs the result.

The goal of this change is to remove that legacy/backup polling mechanism entirely, so device updates rely on UDisks2 signals only (with non-polling error handling).

## Goals

- Remove the polling-based device-event stream API and its usage.
- Ensure the UI still receives device add/remove updates via UDisks2 signals.
- Define clear behavior when signal subscription cannot be established, without introducing periodic polling.
- Update `.copi` documentation to reflect the new behavior.

## Non-Goals

- Changing how drives/partitions are enumerated (e.g., `DriveModel::get_drives()` semantics).
- Adding a new periodic refresh loop (any kind of background polling).
- Broad UI redesign or additional UX work beyond minimal error handling.

## Proposed Approach

1. Remove the polling API from `storage-dbus`:
   - Delete `DiskManager::device_event_stream(interval: Duration)` and associated types/helpers that exist only to support polling.
   - Remove/adjust comments that describe polling or recommend falling back to polling.

2. Update `storage-ui` subscription:
   - Replace “signals-or-polling fallback” logic with “signals-only” logic.
   - If signals subscription fails, log an error and stop the subscription task (or send a one-time message to the app to surface a lightweight error state).

3. Update docs:
   - Update `.copi/architecture.md` device change detection section to describe signals-only behavior.
   - Add an entry to `.copi/spec-index.md` mapping this brief to the new spec/branch.

## User/System Flows

- App start:
  - UI loads current drives via `DriveModel::get_drives()`.
  - UI starts a background subscription task that listens for UDisks2 ObjectManager signals and forwards `Added/Removed` events.

- Device added/removed:
  - A signal is received and converted into `DeviceEvent::Added/Removed`.
  - UI refreshes the drive list.

- Signals unavailable (startup failure):
  - Subscription setup fails.
  - UI does **not** start polling.
  - Error is logged; optional: UI shows a non-blocking “device updates unavailable” state.

## Risks & Mitigations

- Risk: In some environments, signal subscription may fail (DBus connection issues, UDisks2 not present).
  - Mitigation: Fail fast with clear logs; optionally surface a small UI message; rely on manual re-open or explicit refresh flows (non-periodic).

- Risk: Removing the API could affect other consumers.
  - Mitigation: Search for all usages of `device_event_stream` and update compile-time; keep changes localized.

## Acceptance Criteria

- [ ] No polling-based device event code remains (no periodic `GetBlockDevices` diff loop used for eventing).
- [ ] `storage-ui` no longer falls back to polling when signal subscription fails.
- [ ] Device insert/remove updates still occur via UDisks2 signals in normal operation.
- [ ] `.copi/architecture.md` reflects signals-only device change detection.
- [ ] CI quality gates are expected to pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-features`, `cargo test --workspace --all-features`.
