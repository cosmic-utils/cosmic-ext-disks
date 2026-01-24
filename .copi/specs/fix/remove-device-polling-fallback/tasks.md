# fix/remove-device-polling-fallback — Tasks

## Task 1: Remove polling device-event stream from `disks-dbus`

- Scope: Delete the legacy polling-based device event stream and any references to it.
- Files/areas:
  - `disks-dbus/src/disks/manager.rs`
  - Any public exports or docs referencing the polling stream
- Steps:
  - Remove `DiskManager::device_event_stream(interval: Duration)`.
  - Remove `tokio::time::sleep(interval)` loop used to diff `get_block_devices()` results.
  - Update doc comments to remove “fallback to polling” language.
  - Ensure `DeviceEventStream` remains valid for the signal-based stream.
- Test plan:
  - `cargo test -p disks-dbus --all-features`
  - `cargo clippy -p disks-dbus --all-features -- -D warnings`
- Done when:
  - [x] No code compiles that provides a polling-based event stream.
  - [x] `disks-dbus` builds cleanly and clippy passes.

## Task 2: Update UI subscription to be signals-only

- Scope: Remove polling fallback logic in the UI and define behavior on subscription failure.
- Files/areas:
  - `disks-ui/src/app.rs`
- Steps:
  - Replace the `match manager.device_event_stream_signals()` fallback branch.
  - On error, log with context (and stop the stream task), or send a one-time UI message that updates can’t be subscribed.
  - Remove the “Falling back to polling-based device updates” message.
- Test plan:
  - `cargo test -p disks-ui --all-features`
  - Manual smoke: run the app and insert/remove a USB drive; confirm the list updates.
- Done when:
  - [x] UI does not call `device_event_stream(Duration::from_secs(...))`.
  - [x] Signal stream is the only mechanism for device add/remove updates.

## Task 3: Update `.copi` documentation and tracking

- Scope: Make the repo docs match signals-only behavior and register this spec.
- Files/areas:
  - `.copi/architecture.md`
  - `.copi/spec-index.md`
- Steps:
  - Update the “Device change detection” section to describe UDisks2 ObjectManager signal subscription.
  - Add a new row to the spec index for this brief.
- Test plan:
  - N/A (docs)
- Done when:
  - [x] Docs no longer describe polling-based event detection.

## Task 4: Repository quality gates

- Scope: Run standard checks expected by CI.
- Steps:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`
- Done when:
  - [x] All commands pass locally.
