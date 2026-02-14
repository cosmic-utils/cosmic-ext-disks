# fix/file-logging-ui-error-context — Tasks

Branch: `fix/file-logging-ui-error-context`

## Task 1: Add file-based tracing to storage-ui
- Scope: persistent log file output (daily rolling; keep 7 days) + env-configurable filters.
- Likely files:
  - `storage-ui/src/main.rs`
  - new: `storage-ui/src/logging.rs` (or similar)
  - `Cargo.toml` / workspace deps (add `tracing-appender`, possibly `directories`)
- Steps:
  - Add workspace dependency `tracing-appender`.
  - Implement `logging::init()` that installs a `tracing_subscriber` registry with stdout + file layers.
  - Determine log directory using XDG conventions; create it if missing.
  - Add env overrides (`COSMIC_EXT_DISKS_LOG_DIR`/`COSMIC_EXT_DISKS_LOG_FILE`) and `RUST_LOG` support.
  - Configure file appender rotation: split by day; retain 7 days.
  - Ensure ANSI is disabled for file logs.
- Test plan:
  - Run `cargo run -p cosmic-ext-disks` and verify a log file is created.
  - Set `RUST_LOG=trace` and verify verbosity changes.
  - Temporarily set log dir to a read-only location and verify graceful fallback.
- Done when:
  - [x] File log exists at the default path on Linux.
  - [x] App still logs to stdout.
  - [x] No panics if log directory creation fails.

## Task 2: Standardize UI error logging at the point of surfacing
- Scope: ensure every surfaced error is logged with consistent context.
- Likely files:
  - `storage-ui/src/ui/volumes/update/{partition,create,filesystem,encryption,mount_options}.rs`
  - `storage-ui/src/ui/app/update/{drive,image,image/dialogs}.rs`
  - new small helper module (e.g. `storage-ui/src/ui/error.rs` or `storage-ui/src/utils/report.rs`)
- Steps:
  - Inventory all `Err(e) => ShowDialog::Info {..}` and `state.error = Some(...)` sites.
  - Keep pure logging as direct `tracing::info!/warn!/error!` calls (no wrapper).
  - Use a single helper only where we *both* log and construct a dialog: `log_error_and_show_dialog(...)`.
  - Ensure dialog-from-Err sites emit `tracing::error!(?e, operation=..., object_path=..., device=...)`.
  - Keep validation errors as `warn!` (do not flood error logs).
  - Include stable context fields (operation name; volume/block object path if available; drive id/name if available).
- Test plan:
  - Trigger a known failure (e.g., mount failure or formatting without mkfs tools) and confirm:
    - UI dialog appears
    - log file contains an `ERROR` entry with operation context
- Done when:
  - [x] No remaining code paths where a backend `Err(e)` is shown to the user without a corresponding log entry.

## Task 3: Preserve UDisks2 method error messages in storage-dbus operations
- Scope: prevent “The operation failed” from being the only detail.
- Likely files:
  - `storage-dbus/src/disks/ops.rs`
  - potentially `storage-dbus/src/disks/{volume,volume_model,manager}.rs` (call sites)
- Steps:
  - Add a helper for calling UDisks2 methods via `zbus::Proxy::call_method` that:
    - detects `zbus::Error::MethodError(name, msg, ..)`
    - returns an `anyhow` error with `{operation} failed (object_path=...): {name}: {msg}`
  - Migrate key operations to use raw zbus calls (similar to current `partition_delete`):
    - `Block.Format`
    - `Filesystem.Mount` / `Filesystem.Unmount`
    - `Encrypted.Unlock` / `Encrypted.Lock`
    - `PartitionTable.CreatePartitionAndFormat`
  - Add best-effort device identification for better messages (preferred device/device bytes decoding).
  - Ensure passphrases are never included in error strings/log fields.
- Test plan:
  - Induce a DBus method error (e.g., lock while mounted, unmount busy, format failing) and verify the returned error includes method error name+message.
  - Run existing unit tests (`storage-dbus` has tests in `ops.rs`) and add minimal new tests around error formatting if feasible (mock backend returning a method-error-shaped error).
- Done when:
  - [x] User-visible dialogs show a more informative `e` chain for these operations.
  - [x] Logs contain method error details (name/message) for DBus failures.

## Task 4: Documentation + QA checklist
- Scope: make it easy to find logs and validate quickly.
- Likely files:
  - `README.md` (top-level or `storage-ui/README.md`)
- Steps:
  - Document default log file path and env overrides.
  - Add a short “collect logs for bug reports” section.
- Test plan:
  - Verify docs match actual path.
- Done when:
  - [x] README includes log location and overrides.

## Suggested sequence / dependencies
- Do Task 1 first so subsequent tasks can be validated by checking the file logs.
- Then Task 2 (UI) + Task 3 (dbus) in parallel/iterative fashion.
- Finish with Task 4.
