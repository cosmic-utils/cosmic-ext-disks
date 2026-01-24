# Spec: Signal-based device change detection (GAP-008)

Branch: `fix/device-change-detection-signals`

Source:
- Audit: `.copi/audits/2026-01-24T00-37-04Z.md`
- Gap: **GAP-008 — Device change detection is polling-based (1s) instead of signal-based**

## Context
The UI currently refreshes device state using a periodic poll (`device_event_stream(Duration::from_secs(1))`). This introduces constant background work, can miss fast transitions, and makes insert/remove feel laggy.

UDisks2 provides DBus signals via the ObjectManager interface that are designed for tracking dynamic device objects.

## Goals
- Replace 1s polling as the primary mechanism for device insert/remove detection.
- Update device/volume lists promptly on insert/remove.
- Keep a safe fallback (polling) if signal subscription fails at runtime.

## Non-Goals
- Implementing new partition/mount operations.
- Changing UI layout/visual design.
- Overhauling the DBus abstraction beyond what is needed for eventing.

## Proposed Approach
- In the DBus layer, add a signal-driven event stream based on `org.freedesktop.DBus.ObjectManager`:
  - Subscribe to `InterfacesAdded` and `InterfacesRemoved` from UDisks2.
  - Map those signals into the existing “device changed” event type used by the UI.
- In the UI layer, switch the event source to the new signal-based stream.
- Keep polling as a fallback:
  - If subscribing to signals fails (e.g., bus permission/connection issues), log and fall back to the existing polling stream.
  - Consider a slower polling interval (e.g., 5–10s) as the fallback to reduce load.

Likely touch points (by evidence from audit):
- `disks-ui/src/app.rs` (current `device_event_stream(Duration::from_secs(1))` usage)
- `disks-dbus/src/disks/manager.rs` (current polling loop)

## User/System Flows
- Device inserted → DBus `InterfacesAdded` received → manager updates internal cache → UI list refreshes.
- Device removed → DBus `InterfacesRemoved` received → manager updates internal cache → UI list refreshes.
- Signal subscription fails → fallback polling stream drives updates.

## Risks & Mitigations
- DBus signal handling differs across environments
  - Mitigation: keep polling fallback; add clear logs when in fallback.
- Race conditions between initial enumeration and signal stream startup
  - Mitigation: perform an initial full refresh, then start listening; ensure UI can handle duplicate/rapid updates.

## Acceptance Criteria
- [x] Device insert/remove updates occur promptly without polling.
- [x] If signals are unavailable, the app continues functioning via fallback polling.
- [x] No regressions to existing device list rendering.

## Implementation Notes
- Primary mechanism is UDisks2 ObjectManager signals (InterfacesAdded/Removed), filtered to `org.freedesktop.UDisks2.Block`.
- UI falls back to polling (`Duration::from_secs(10)`) if signal subscription fails.
