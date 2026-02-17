# Implementation Plan: Service Hardening

**Branch**: `001-service-hardening` | **Date**: 2026-02-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-service-hardening/spec.md`

## Summary

This feature addresses performance issues in the COSMIC storage application by implementing persistent D-Bus connections at **TWO layers**:

1. **Layer 1 (storage-ui → storage-service)**: The UI app currently creates a new D-Bus connection for each client instance. We will implement a shared connection pool.

2. **Layer 2 (storage-dbus → UDisks2)**: The storage-dbus library's `get_disks_with_volumes()` creates a fresh connection on every call (called 9+ times in service). We will implement connection caching in DiskManager.

Additionally, this feature adds system path protection for unmount operations and consolidates filesystem tool detection in the service.

## Technical Context

**Language/Version**: Rust (edition 2024, stable channel)
**Primary Dependencies**: zbus 5.x (D-Bus), udisks2 crate, libcosmic (UI), tokio (async)
**Storage**: N/A (D-Bus mediated access to UDisks2)
**Testing**: `cargo test --workspace --all-features`, integration tests via D-Bus
**Target Platform**: Linux only (systemd-based distributions)
**Project Type**: Workspace with 6 crates (ui, service, dbus, common, sys, btrfs)
**Performance Goals**: <3s startup, <1s event response, <500ms disk enumeration
**Constraints**: Must run as root for storage-service; Polkit for privileged operations
**Scale/Scope**: Desktop application, single-user, local disk management

### Current Performance Problems Identified

| Layer | Issue | Location | Impact |
|-------|-------|----------|--------|
| storage-ui → storage-service | Each client creates `Connection::system()` | `storage-ui/src/client/*.rs:77-79` | Multiple connections during startup |
| storage-dbus → UDisks2 | `get_disks_with_volumes()` creates fresh connection | `storage-dbus/src/disk/discovery.rs:417` | 9+ connections per service operation |

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. Data Safety First** | ✅ PASS | System path protection prevents accidental system damage |
| **II. Modular Crate Architecture** | ✅ PASS | Changes are localized to specific crates (ui, dbus, service) |
| **III. Quality Gates** | ✅ PASS | All changes will pass `cargo test/clippy/fmt` |
| **IV. Evidence-Based Design** | ✅ PASS | Performance issues identified via code exploration |
| **V. Linux System Integration** | ✅ PASS | Uses standard D-Bus/UDisks2 APIs |

**Gate Result**: PASS - No violations to justify.

## Project Structure

### Documentation (this feature)

```text
specs/001-service-hardening/
├── spec.md              # Feature specification (updated with Layer 2 requirements)
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (internal API contracts)
└── tasks.md             # Phase 2 output
```

### Source Code (workspace structure)

```text
storage-ui/
├── src/
│   ├── client/          # D-Bus clients - MODIFY: add connection sharing
│   │   ├── mod.rs
│   │   ├── disks.rs     # DisksClient - USE shared connection
│   │   ├── filesystems.rs
│   │   └── partitions.rs
│   ├── models/          # UI models
│   │   ├── app.rs       # AppModel - ADD: SharedConnection initialization
│   │   └── load.rs      # MODIFY: use shared connection
│   └── connection.rs    # NEW: SharedConnection singleton

storage-service/
├── src/
│   ├── main.rs          # Service entry - ADD: FSTools feature detection
│   ├── service.rs       # D-Bus service - ADD: capabilities property
│   ├── disks.rs         # MODIFY: system path protection check
│   └── protected_paths.rs # NEW: protected path validation logic

storage-dbus/
├── src/
│   ├── disk/
│   │   ├── mod.rs
│   │   ├── discovery.rs # MODIFY: use cached connection from DiskManager
│   │   └── manager.rs   # MODIFY: expose cached connection
│   └── connection.rs    # NEW: Connection caching utilities

storage-common/
├── src/
│   ├── capabilities.rs  # NEW: ServiceCapabilities type
│   └── protected.rs     # NEW: ProtectedPath types
```

**Structure Decision**: Changes are localized to existing crate structure. New modules added for connection management (`connection.rs`) and capabilities (`capabilities.rs`, `protected.rs`).

## Complexity Tracking

> No violations - Constitution Check passed all gates.

## Implementation Phases

### Phase 0: Layer 2 Connection Caching (storage-dbus → UDisks2) - HIGHEST PRIORITY

**Rationale**: This layer has the most impact - `get_disks_with_volumes()` is called 9+ times per operation.

**Changes**:
1. Add `connection: Arc<Connection>` field to `DiskManager`
2. Expose via `manager.connection()` method
3. Update `get_disks_with_volumes()` to accept `&DiskManager` parameter
4. Update all call sites in `storage-service`

### Phase 1: Layer 1 Connection Sharing (storage-ui → storage-service)

**Rationale**: Reduces UI startup time and improves responsiveness.

**Changes**:
1. Create `storage-ui/src/client/connection.rs` with `OnceLock<Connection>` singleton
2. Update all client `new()` methods to use `shared_connection()`
3. Export module in `client/mod.rs`

### Phase 2: Protected System Paths

**Rationale**: Safety-critical feature to prevent accidental system damage.

**Changes**:
1. Create `storage-service/src/protected_paths.rs` with `PROTECTED_SYSTEM_PATHS` constant
2. Add `is_protected_path()` function with canonical path matching
3. Update `unmount()` method to check before `kill_processes`

### Phase 3: FSTools Consolidation

**Rationale**: Maintainability improvement - single source of truth for tool detection.

**Changes**:
1. Add `FilesystemToolInfo` to `storage-common`
2. Enhance `FilesystemsHandler` with comprehensive tool detection
3. Add `get_filesystem_tools()` D-Bus method
4. Add client method in `storage-ui`
5. Deprecate `storage-ui/src/utils/fs_tools.rs`

## Dependencies Between Phases

```
Phase 0 (Layer 2) ──┐
                    ├──> Phase 3 (benefits from both connection improvements)
Phase 1 (Layer 1) ──┘

Phase 2 (Protected Paths) ──> Independent
```

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| App startup time | ~5-10s | <3s |
| Disk enumeration | ~2-3s each | <500ms first, <200ms cached |
| UI event response | ~1-2s | <500ms |
| D-Bus connections per operation | 10+ | 1-2 |

---

## APPENDIX B: Polkit Authorization & User Context Passthrough

*Added during implementation: Security audit revealed critical vulnerabilities requiring immediate attention.*

### Security Issues Identified

1. **Polkit Bypass**: `check_polkit_auth()` uses `connection.unique_name()` which returns the service's own name (root), not the caller's. This means ALL authorization checks pass because root is always authorized.

2. **UDisks2 User Context Loss**: When the service calls UDisks2, UDisks2 sees the request coming from root. Mount points appear under `/run/media/root/` with root-owned files, inaccessible to the actual user.

### Revised Implementation Phases

The original phases (0-3) remain valid but security fixes take priority:

#### Phase 0.5: Authorization Macro Infrastructure (CRITICAL - Do First)

**Rationale**: This is a complete security bypass. Any user can perform any destructive operation without authentication.

**Changes**:

1. **Create `storage-service-macros` crate**:
   ```text
   storage-service-macros/
   ├── Cargo.toml          # proc-macro = true
   └── src/
       └── lib.rs          # #[authorized_interface()] macro
   ```

2. **Define `CallerInfo` struct in `storage-common`**:
   ```rust
   pub struct CallerInfo {
       pub uid: u32,
       pub username: Option<String>,
       pub sender: String,
   }
   ```

3. **Implement the macro**:
   - Wraps `#[zbus::interface]` functionality
   - Auto-injects `#[zbus(header)]` and `#[zbus(connection)]`
   - Extracts caller info before method body
   - Performs Polkit check with correct subject
   - Injects `CallerInfo` parameter into method

**Files**:
- `storage-service-macros/Cargo.toml` (NEW)
- `storage-service-macros/src/lib.rs` (NEW)
- `storage-common/src/caller.rs` (NEW)
- `storage-common/src/lib.rs` (MODIFY: export caller module)

---

#### Phase 0.6: Migrate Service Methods to Authorized Macro (CRITICAL)

**Rationale**: All existing authorization is broken. Must migrate all methods.

**Changes by File**:

| File | Methods to Migrate | Priority |
|------|-------------------|----------|
| `storage-service/src/filesystems.rs` | `mount`, `unmount`, `format`, `set_label`, `take_ownership`, `check`, `repair` | P1 |
| `storage-service/src/partitions.rs` | `create_partition`, `delete_partition`, `resize_partition`, `set_partition_type`, `set_partition_name`, `set_partition_flags` | P1 |
| `storage-service/src/luks.rs` | `unlock_luks`, `lock_luks`, `format_luks`, `change_passphrase` | P1 |
| `storage-service/src/btrfs.rs` | `create_subvolume`, `delete_subvolume`, `create_snapshot` | P2 |
| `storage-service/src/zram.rs` | `create_zram`, `destroy_zram` | P2 |
| `storage-service/src/disks.rs` | `eject`, `power_off` | P2 |

**Migration Pattern**:

```rust
// BEFORE (broken)
async fn mount(&self, device: String, ...) -> zbus::fdo::Result<String> {
    check_polkit_auth(connection, "action.id").await?;  // Checks against root!
    // ...
}

// AFTER (fixed)
#[authorized_interface(action = "org.cosmic.ext.storage-service.mount")]
async fn mount(&self, caller: CallerInfo, device: String, ...) -> zbus::fdo::Result<String> {
    // Authorization already checked against actual caller
    // caller.uid and caller.username available
}
```

---

#### Phase 0.7: UDisks2 User Context Passthrough (CRITICAL)

**Rationale**: Users cannot access their mounted filesystems.

**Changes**:

1. **Update `mount_filesystem()` signature**:
   ```rust
   // storage-dbus/src/filesystem/mount.rs
   pub async fn mount_filesystem(
       device_path: &str,
       mount_point: &str,
       options: MountOptions,
       caller_uid: Option<u32>,  // NEW
   ) -> Result<String, DiskError>
   ```

2. **Add username resolution**:
   ```rust
   fn get_username_from_uid(uid: u32) -> Option<String> {
       unsafe { libc::getpwuid(uid) }
           .as_ref()
           .and_then(|pw| std::ffi::CStr::from_ptr((*pw).pw_name).to_str().ok())
           .map(|s| s.to_string())
   }
   ```

3. **Pass `as-user` and `uid` to UDisks2**:
   ```rust
   if let Some(uid) = caller_uid {
       if let Some(username) = get_username_from_uid(uid) {
           opts.insert("as-user", Value::from(username));
           opts.insert("uid", Value::from(uid));
       }
   }
   ```

4. **Update service call sites** to pass `caller.uid`:

**Files**:
- `storage-dbus/src/filesystem/mount.rs` (MODIFY)
- `storage-service/src/filesystems.rs` (MODIFY: pass caller.uid)

---

### Updated Dependencies Between Phases

```
Phase 0.5 (Macro Infrastructure)
         │
         ▼
Phase 0.6 (Migrate Methods) ──────┐
         │                        │
         ▼                        ▼
Phase 0.7 (User Passthrough)   Phase 0 (Layer 2 Caching)
         │                        │
         └────────┬───────────────┘
                  ▼
            Phase 1 (Layer 1 Sharing)
                  │
                  ▼
            Phase 2 (Protected Paths)
                  │
                  ▼
            Phase 3 (FSTools)
```

---

### Updated Project Structure

```text
storage-service-macros/           # NEW CRATE
├── Cargo.toml                    # proc-macro = true
└── src/
    └── lib.rs                    # #[authorized_interface()] macro

storage-common/
├── src/
│   ├── caller.rs                 # NEW: CallerInfo struct
│   ├── capabilities.rs           # ServiceCapabilities
│   └── protected.rs              # ProtectedPath types

storage-service/
├── Cargo.toml                    # MODIFY: add storage-service-macros dep
└── src/
    ├── filesystems.rs            # MODIFY: use #[authorized_interface]
    ├── partitions.rs             # MODIFY: use #[authorized_interface]
    ├── luks.rs                   # MODIFY: use #[authorized_interface
    ├── btrfs.rs                  # MODIFY: use #[authorized_interface]
    ├── zram.rs                   # MODIFY: use #[authorized_interface]
    ├── disks.rs                  # MODIFY: use #[authorized_interface]
    └── auth.rs                   # DEPRECATE: check_polkit_auth()

storage-dbus/
└── src/
    └── filesystem/
        └── mount.rs              # MODIFY: accept caller_uid, use as-user
```

---

### Additional Success Criteria

| ID | Criterion | Verification |
|----|-----------|--------------|
| SC-010 | All destructive methods require proper Polkit auth | Attempt as unprivileged user |
| SC-011 | Polkit password prompts appear when required | Manual testing |
| SC-012 | Mount points created under `/run/media/<username>/` | Mount and check path |
| SC-013 | Files on mounted FAT/NTFS owned by mounting user | `ls -la` on mount |
| SC-014 | `check_polkit_auth()` deprecated or removed | Code review |
| SC-015 | No authorization uses `connection.unique_name()` | Code search |
