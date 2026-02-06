# File Logging + UI Error Reporting + UDisks2 Error Context

Branch: `fix/file-logging-ui-error-context`
Source: N/A (brief; 2026-02-04)
Related: `.copi/audits/2026-01-25T23-24-44Z.md` (GAP-011 in that audit: logging/error reporting)

## Context
Today the UI initializes `tracing_subscriber::fmt().init()` (stdout only). When users report bugs, we often lack persistent logs.

Separately, several UI flows surface errors to the user via `ShowDialog::Info` or per-dialog `state.error`, but do not always emit a corresponding structured log entry. This makes it hard to correlate user-visible failures with root causes.

Finally, some DBus/UDisks2 operation failures arrive as low-signal messages like “The operation failed”. We already worked around this for `Partition.Delete` by using raw zbus calls to preserve the original method error (name + message). Similar loss of context can happen for other UDisks2 operations.

## Goals
- Persist application logs to a file by default on Linux.
- Ensure that every error that is surfaced in the UI is also logged (with useful context) at the moment it is surfaced.
- Improve error context for UDisks2 operations (method + object path + best-effort device identifier), avoiding “operation failed” as the only detail.

## Non-Goals
- Changing user-facing error text wording beyond adding missing context (e.g., keeping dialog titles and basic formatting stable).
- Reworking the overall error-handling architecture or introducing a new UI error framework.
- Fixing unrelated issues (e.g., app-id mismatches between desktop/metainfo/config), unless required for log paths.

## Proposed Approach
### 1) File logging in `disks-ui`
- Add a small logging bootstrap module (e.g. `disks-ui/src/logging.rs`) that configures `tracing_subscriber` as a registry with:
  - an `EnvFilter` (respects `RUST_LOG`, provides a sensible default)
  - a stdout `fmt` layer (current behavior)
  - a file `fmt` layer writing to a rolling log file (daily rotation; keep 7 days)
- Use `tracing_appender` for non-blocking, rolling file output.
- Choose an XDG-compliant default location:
  - Prefer `$XDG_STATE_HOME/cosmic-ext-disks/logs/` (fallback `~/.local/state/cosmic-ext-disks/logs/`)
  - File name: `cosmic-ext-disks.log` (rolling daily; keep 7 days)
- Make the log path overrideable for debugging:
  - `COSMIC_EXT_DISKS_LOG_DIR` (directory)
  - `COSMIC_EXT_DISKS_LOG_FILE` (explicit file path)
- Ensure file logs are ANSI-free and include timestamps + targets.

### 2) UI error surfacing always logs
- Identify all places where errors are surfaced:
  - `Message::Dialog(Box::new(ShowDialog::Info { .. }))`
  - setting `state.error = Some(...)` where that value is displayed
- Introduce a tiny helper in the UI layer (e.g. `disks-ui/src/ui/error.rs` or `disks-ui/src/utils/logging.rs`) to standardize:
  - `log_error_and_show_dialog(title, err, context_fields...)`
- Prefer direct `tracing::info!/warn!/error!` calls in-line for pure logging.
- The only exception (allowed wrapper) is `log_error_and_show_dialog(...)` since it couples logging with the UI error surface.
- Rules:
  - User input validation errors: `tracing::warn!` (or `debug!`), not `error!`
  - Backend/DBus/UDisks errors: `tracing::error!` with `?err` and structured fields (volume id/path, operation name)
- Target the modules with the highest concentration of `ShowDialog::Info` creation:
  - `disks-ui/src/ui/volumes/update/*.rs`
  - `disks-ui/src/ui/app/update/*.rs`

### 3) Preserve UDisks2 method error details
- Extend the “raw zbus proxy” approach beyond `Partition.Delete` for operations that users trigger and that may currently lose the message:
  - `Filesystem.Mount`, `Filesystem.Unmount`
  - `Block.Format`
  - `Encrypted.Unlock`, `Encrypted.Lock`
  - `PartitionTable.CreatePartitionAndFormat`
- Implement a shared helper in `disks-dbus` (likely in `disks-dbus/src/disks/ops.rs`) for calling a UDisks2 method that:
  - catches `zbus::Error::MethodError(name, msg, ..)`
  - returns an `anyhow` error containing `name` + `msg` + operation + object path
  - optionally enriches with best-effort device identifiers (preferred device, device, etc.) similar to the existing `partition_delete` logic
- Ensure sensitive values are never logged:
  - passphrases must never appear in logs, even at `debug` (keep/extend redaction patterns like `RedactedString`)

## User/System Flows
- User runs the app → logs appear in the log directory without any configuration.
- User hits a failure (format, mount, resize, unlock, delete, create) → UI shows an error dialog and a structured log entry is written with operation + object path + underlying DBus error.
- Developer asks user for logs → user can attach a single log file from the known location.

## Risks & Mitigations
- **Log directory creation failures (permissions, sandboxing):** fall back to stdout-only and emit a one-time warning.
- **Performance impact:** use non-blocking file appender.
- **Sensitive data leakage:** keep redaction strict for passphrases; avoid dumping full option maps for format/unlock.
- **Inconsistent context fields:** use helpers to keep logging consistent across modules.

## Acceptance Criteria
- [x] App writes logs to an XDG-compliant file location by default.
- [x] Log file path is documented (and/or easily discoverable) and can be overridden via env vars.
- [x] Every user-visible error dialog created from an `Err(...)` also emits a `tracing::error!` (with operation context).
- [x] At least the primary UDisks2 user-triggered operations preserve `MethodError` name + message in the error chain.
- [x] No passphrases are written to logs.
- [x] Logs are split by day and retain the last 7 days by default.
- [x] `cargo test --workspace --all-features`, `cargo clippy --workspace --all-features`, and `cargo fmt --all --check` still pass.
