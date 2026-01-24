# fix/device-change-detection-signals — Tasks

Source: `.pigentic/audits/2026-01-24T00-37-04Z.md` (GAP-008)

## Task 1: Add DBus signal event stream (dbus layer)
- Scope: Provide a signal-driven “device changed” stream based on UDisks2 ObjectManager signals.
- Files/areas:
  - `disks-dbus/src/disks/manager.rs`
  - (Potentially) `disks-dbus/src/disks/mod.rs` and related types
- Steps:
  - Identify the current polling-based event API and its consumer(s).
  - Add a new stream/source that subscribes to `InterfacesAdded` and `InterfacesRemoved`.
  - Translate received signals into the internal “device changed” event.
  - Ensure clean shutdown/cancellation when dropping the stream.
- Test plan:
  - Manual: run the UI, plug/unplug a USB drive, verify list updates immediately.
  - Basic logging: confirm the signal handler triggers on events.
- Done when:
  - [x] Signal subscription exists and emits events on add/remove.
  - [x] Existing polling mechanism still compiles and can be used as fallback.

## Task 2: Wire UI to prefer signals with fallback polling
- Scope: Switch UI to use signal-based eventing primarily.
- Files/areas:
  - `disks-ui/src/app.rs`
- Steps:
  - Replace `device_event_stream(Duration::from_secs(1))` usage with the new signal-based stream.
  - Add error handling to fall back to polling if subscription fails.
  - Consider increasing fallback poll interval to reduce load.
- Test plan:
  - Manual: verify insert/remove works; verify app still updates if DBus signal subscription is forced to fail.
- Done when:
  - [x] UI uses signals by default.
  - [x] Polling is only used as fallback.

## Task 3: Document behavior + troubleshooting
- Scope: Capture how eventing works and how to debug fallback.
- Files/areas:
  - `README.md` (or `disks-ui/README.md` if more appropriate)
- Steps:
  - Add a short note describing signal-based detection and the polling fallback.
  - Note any env vars/logging hints if applicable.
- Test plan:
  - N/A (documentation-only)
- Done when:
  - [x] Docs mention signals + fallback and where to look for logs.
