# Usage Scan Parallelism Preset Implementation Plan

**Goal:** Add a persisted Settings option for usage scan parallelism (`Low/Balanced/High`) and pass it through usage scan initialization so service maps presets to concrete scanner thread counts.

**Architecture:** Keep policy mapping in service, persistence in UI config, and scanner input as concrete `threads` only.

**Tech Stack:** Rust workspace (`storage-common`, `storage-service`, `storage-sys`, `storage-ui`), COSMIC settings/config, zbus client/service APIs.

---

## Task 1: Add shared preset enum and request field

**Files:**
- Modify: `storage-common/src/usage_scan.rs`
- Modify: `storage-common/src/lib.rs`

**Steps:**
1. Add `UsageScanParallelismPreset` enum with variants `Low`, `Balanced`, `High`.
2. Add `parallelism_preset: UsageScanParallelismPreset` to `UsageScanRequest` (or current usage-scan request payload type in use).
3. Derive serde traits and ensure stable serialized naming.
4. Add unit roundtrip test for enum/request serialization.

**Validation:**
- `cargo test -p storage-common usage_scan_request_and_delete_result_roundtrip -v`

---

## Task 2: Extend UI config and settings control

**Files:**
- Modify: `storage-ui/src/config.rs`
- Modify: `storage-ui/src/ui/app/message.rs`
- Modify: `storage-ui/src/ui/app/update/mod.rs`
- Modify: `storage-ui/src/views/settings.rs`

**Steps:**
1. Add persisted config field `usage_scan_parallelism` with default `Balanced`.
2. Add app message variant for settings update.
3. In update reducer, persist value via existing cosmic config helper (`write_entry`).
4. Render settings control in Settings pane with exactly three values: Low/Balanced/High.

**Validation:**
- `cargo check -p cosmic-ext-storage`

---

## Task 3: Pass preset through scan initialization

**Files:**
- Modify: `storage-ui/src/ui/app/update/mod.rs`
- Modify: `storage-ui/src/client/filesystems.rs`

**Steps:**
1. Extend usage scan message/request path to include current config preset.
2. Extend client proxy and typed client method signature to pass preset.
3. Keep existing behavior for other scan controls unchanged.

**Validation:**
- `cargo check -p cosmic-ext-storage`

---

## Task 4: Extend service API and map preset to threads

**Files:**
- Modify: `storage-service/src/filesystems.rs`
- Modify: `storage-sys/src/usage/types.rs` (only if needed for helper integration)

**Steps:**
1. Extend `get_usage_scan` DBus method signature to accept preset.
2. Add helper mapping function from preset + CPU count to thread count:
   - `Low`: `max(1, ceil(n/4))`
   - `Balanced`: `max(1, ceil(n/2))`
   - `High`: `max(1, n)`
3. Set `ScanConfig.threads = Some(mapped_threads)`.
4. Keep scanner implementation unchanged except receiving thread count.

**Validation:**
- `cargo check -p storage-service -p cosmic-ext-storage-storage-sys`

---

## Task 5: Add focused tests for mapping and persistence

**Files:**
- Modify: `storage-service/src/filesystems.rs` (or nearby module tests)
- Modify: `storage-ui` tests near config/update where practical

**Steps:**
1. Add service mapping tests for representative CPU counts (1, 2, 8, odd count).
2. Add config roundtrip or reducer test proving preset updates persist path.
3. Add compile-level guard where runtime tests are not practical.

**Validation:**
- `cargo test -p storage-service -v`
- `cargo check -p cosmic-ext-storage`

---

## Task 6: Final verification

**Steps:**
1. Run targeted checks:
   - `cargo check -p storage-common -p storage-service -p cosmic-ext-storage-storage-sys -p cosmic-ext-storage`
2. Manual smoke:
   - Change setting Low/Balanced/High.
   - Restart app and verify persistence.
   - Trigger usage refresh and confirm scan succeeds for each preset.

---

## Notes
- Keep UX minimal and scoped to Settings pane only.
- Do not add extra pages/modals/tooltips unless explicitly requested.
- Preserve existing usage result schema.
