# Research: Audit 2026-02-14 Gap Remediation

**Date**: 2026-02-14
**Source**: `.copi/audits/2026-02-14T17-00-36Z.md`

## Research Summary

This document consolidates findings from the architecture audit and establishes implementation approaches for each gap.

---

## GAP-002: Optional Connection Anti-Pattern

### Decision
Make `Connection` required (not `Option<Connection>`) in VolumeNode and VolumeModel.

### Rationale
1. Connection is always `Some` in production use
2. The `Option` wrapper provides false flexibility
3. Every operation method unwraps it, causing potential panic
4. Tests should use test connections, not `None`

### Alternatives Considered

| Approach | Pros | Cons | Decision |
|----------|------|------|----------|
| Required Connection | Compile-time safety, clear semantics | Requires constructor changes | ✅ CHOSEN |
| Arc<Connection> | Shared ownership, never None | More indirection | Alternative if borrow issues arise |
| Separate Operations struct | Clean separation | More types to maintain | Consider for future refactor |

### Implementation Approach

```rust
// Before (problematic)
pub struct VolumeNode {
    connection: Option<Connection>,
}

// After (safe)
pub struct VolumeNode {
    connection: Connection,
}

impl VolumeNode {
    pub async fn from_block_object(
        connection: &Connection,  // Required parameter
        // ...
    ) -> Result<Self> {
        Ok(Self {
            connection: connection.clone(),
            // ...
        })
    }
}
```

### Files Affected
- `storage-dbus/src/disks/volume.rs`
- `storage-dbus/src/disks/volume_model/partition.rs`
- `storage-dbus/src/disks/volume_model/filesystem.rs`

---

## GAP-003: Blocking Runtime Creation in Clone

### Decision
Remove `Clone` impl that creates blocking runtime; use `Arc` for shared ownership.

### Rationale
1. Creating runtime is expensive (~1ms)
2. `block_on()` from async context can deadlock
3. Tokio best practice: 1 runtime per process
4. Clients are cheap to Arc-clone

### Alternatives Considered

| Approach | Pros | Cons | Decision |
|----------|------|------|----------|
| Arc-wrap clients | Cheap clone, no runtime needed | Shared mutability concerns | ✅ CHOSEN (with GAP-006) |
| Remove Clone | Simplest | May break existing code | Consider if Arc fails |
| Lazy client init | Deferred cost | More complex | Alternative approach |

### Implementation Approach

```rust
// Before (problematic)
impl Clone for UiDrive {
    fn clone(&self) -> Self {
        let rt = tokio::runtime::Runtime::new().expect("...");
        let client = rt.block_on(DisksClient::new()).expect("...");
        // ...
    }
}

// After (with GAP-006 ClientPool)
pub struct UiDrive {
    pub disk: DiskInfo,
    pub volumes: Vec<UiVolume>,
    pub partitions: Vec<PartitionInfo>,
    clients: Arc<ClientPool>,  // Arc-clone is cheap
}

// Clone is now trivial (derived or simple field clone)
```

### Files Affected
- `storage-ui/src/models/ui_drive.rs`
- `storage-ui/src/models/helpers.rs`

---

## GAP-004: Missing Parent Path Population

### Decision
Track parent device during tree flattening and populate `parent_path`.

### Rationale
1. VolumeInfo.parent_path is used by UI for tree construction
2. Current TODO leaves it as `None`
3. Breaks hierarchy display for LUKS, LVM, BTRFS

### Implementation Approach

```rust
fn flatten_volumes(
    node: &disks_dbus::VolumeNode,
    parent_device: Option<String>,
    output: &mut Vec<storage_models::VolumeInfo>,
) {
    let mut vol_info: storage_models::VolumeInfo = node.clone().into();

    // SET PARENT PATH
    vol_info.parent_path = parent_device.clone();

    // Recurse with THIS device as parent for children
    let current_device = vol_info.device_path.clone();
    for child in &node.children {
        flatten_volumes(child, current_device.clone(), output);
    }

    vol_info.children.clear();
    output.push(vol_info);
}
```

### Files Affected
- `storage-dbus/src/disks/volume.rs` (From<VolumeNode> for VolumeInfo)
- `storage-service/src/disks.rs` (flatten_volumes function)

---

## GAP-005: Excessive Unwrap/Expect Usage

### Decision
Replace production unwrap/expect with proper error propagation.

### Rationale
1. 100+ instances found across codebase
2. Many in hot paths (called per-operation)
3. Constitution Principle I violation

### Categories and Treatment

| Category | Treatment | Priority |
|----------|-----------|----------|
| Test-only | Keep (acceptable) | N/A |
| String parsing | Replace with ok_or_else | High |
| Mutex locks | Handle poisoning or expect with context | High |
| Path manipulation | Replace with ok_or_else | Medium |
| Static initialization | Keep with descriptive message | Low |

### Pattern for Replacement

```rust
// Before
let unit = pretty.split_whitespace().last().unwrap();

// After
let unit = pretty.split_whitespace().last()
    .ok_or_else(|| BtrfsError::ParseError("Invalid size format".into()))?;
```

### Files Affected
- `storage-models/src/common.rs`
- `storage-btrfs/src/subvolume.rs`
- `storage-dbus/src/disks/ops.rs`
- `storage-ui/src/logging.rs`

---

## GAP-006: Unclear Client Ownership Model

### Decision
Implement `ClientPool` pattern with Arc sharing.

### Rationale
1. Current pattern mixes per-operation and per-struct ownership
2. Unclear lifecycle causes bugs
3. zbus Connection may be cached internally, but explicit sharing is clearer

### Implementation Approach

```rust
// In storage-ui/src/client/pool.rs (new file)
pub struct ClientPool {
    disks: DisksClient,
    partitions: PartitionsClient,
    filesystems: FilesystemsClient,
    luks: LuksClient,
    lvm: LvmClient,
    btrfs: BtrfsClient,
    image: ImageClient,
}

impl ClientPool {
    pub async fn new() -> Result<Self, ClientError> {
        Ok(Self {
            disks: DisksClient::new().await?,
            partitions: PartitionsClient::new().await?,
            filesystems: FilesystemsClient::new().await?,
            luks: LuksClient::new().await?,
            lvm: LvmClient::new().await?,
            btrfs: BtrfsClient::new().await?,
            image: ImageClient::new().await?,
        })
    }
}

// In AppModel
pub struct AppModel {
    core: Core,
    clients: Arc<ClientPool>,
    // ...
}

// In UiDrive
pub struct UiDrive {
    pub disk: DiskInfo,
    pub volumes: Vec<UiVolume>,
    pub partitions: Vec<PartitionInfo>,
    clients: Arc<ClientPool>,  // Shared reference
}
```

### Files Affected
- `storage-ui/src/app.rs`
- `storage-ui/src/models/ui_drive.rs`
- `storage-ui/src/models/helpers.rs`
- New file: `storage-ui/src/client/pool.rs`

---

## GAP-007: JSON Serialization Not Type-Safe

### Decision
Accept current approach but add integration tests for serialization contracts.

### Rationale
1. JSON-over-D-Bus is pragmatic for complex types
2. zvariant approach has complexity trade-offs
3. Integration tests catch schema mismatches

### Implementation
Add serialization round-trip tests in `tests/integration/serialization.rs`.

---

## GAP-008: No Validation in Partition Creation

### Decision
Add comprehensive input validation before UDisks2 calls.

### Rationale
1. Current errors are cryptic UDisks2 messages
2. Users don't understand what went wrong
3. Validation errors are actionable

### Validation Rules

| Field | Rule | Error Message |
|-------|------|---------------|
| disk | Must start with `/dev/` | "Device path must start with /dev/" |
| size | Must be > 0 | "Partition size must be greater than zero" |
| offset | Must be 1MB aligned (GPT) | "Offset must be aligned to {alignment} bytes" |
| type_id | Valid GUID (GPT) or hex byte (DOS) | "Invalid GPT type GUID" or "Invalid DOS partition type" |
| offset + size | Must be <= disk size | "Partition would exceed disk size ({size})" |

### Files Affected
- `storage-service/src/partitions.rs`

---

## GAP-009: Conversions Module is Temporary Workaround

### Decision
Verify Phase 3A completion; if complete, delete conversions.rs.

### Rationale
1. Module exists to convert DriveModel → DiskInfo
2. Should have been removed after Phase 3A
3. If still needed, Phase 3A is incomplete

### Files Affected
- `storage-service/src/conversions.rs` (delete)

---

## GAP-010: TODO Comments Without Context

### Decision
Link all TODOs to GitHub issues or mark as DEFERRED.

### Genuine TODOs Found

| Location | TODO | Action |
|----------|------|--------|
| volume.rs:651 | parent_path | Covered by GAP-004 |
| partition.rs:195 | flag checking methods | Create issue or mark DEFERRED |
| encryption.rs:126 | get_encryption_options_settings | Create issue or mark DEFERRED |
| encryption.rs:301 | take_ownership operation | Create issue or mark DEFERRED |

---

## GAP-011: Mutex Lock Panics Not Handled

### Decision
Replace `.lock().unwrap()` with `.lock().expect("context")` or handle poisoning.

### Rationale
1. 30+ instances in MockDiskBackend
2. Poisoned mutex causes cascading test failures
3. Poor error messages

### Pattern

```rust
// Before
*self.mount_result.lock().unwrap() = res;

// After (Option A)
*self.mount_result.lock()
    .expect("MockDiskBackend mount_result mutex poisoned") = res;

// After (Option B - handle poisoning)
match self.mount_result.lock() {
    Ok(mut guard) => *guard = res,
    Err(poison) => {
        tracing::warn!("Mutex poisoned, clearing: {}", poison);
        *poison.into_inner() = res;
    }
}
```

### Files Affected
- `storage-dbus/src/disks/ops.rs`

---

## GAP-012: No Error Context in Client Error Conversions

### Decision
Preserve D-Bus error names and source context in ClientError.

### Implementation Approach

```rust
#[derive(Error, Debug, Clone)]
pub enum ClientError {
    #[error("D-Bus method call error: {message} (dbus_name: {dbus_name:?})")]
    MethodCall {
        message: String,
        dbus_name: Option<String>,
    },
    // ...
}

impl From<zbus::Error> for ClientError {
    fn from(err: zbus::Error) -> Self {
        match &err {
            zbus::Error::FDO(fdo_err) => {
                let dbus_name = fdo_err.name().map(|n| n.to_string());
                // ...
            }
            // ...
        }
    }
}
```

### Files Affected
- `storage-ui/src/client/error.rs`

---

## GAP-013: No Timeout Handling for Long-Running Operations

### Decision
Add configurable timeouts and progress signal subscription.

### Implementation Approach

```rust
pub async fn format_with_progress(
    &self,
    device: &str,
    fs_type: &str,
    label: &str,
    options: Option<&str>,
    progress_callback: impl Fn(f64) + Send + 'static,
) -> Result<(), ClientError> {
    // Subscribe to FormatProgress signal
    let mut stream = self.proxy.receive_format_progress().await?;

    tokio::spawn(async move {
        while let Some(signal) = stream.next().await {
            if let Ok(args) = signal.args() {
                progress_callback(args.progress);
            }
        }
    });

    // Start format with timeout
    timeout(
        Duration::from_secs(600),
        self.proxy.format(device, fs_type, label, options.unwrap_or("{}"))
    )
    .await
    .map_err(|_| ClientError::Timeout("Format timed out after 10 minutes".into()))??;

    Ok(())
}
```

### Files Affected
- `storage-ui/src/client/filesystems.rs`
- `storage-ui/src/client/image.rs`

---

## GAP-014: Storage-Service Not Checking If Service Is Already Running

### Decision
Add startup check with clear error message.

### Implementation

```rust
async fn is_service_already_running() -> Result<bool> {
    let conn = Connection::system().await?;
    match conn.request_name("org.cosmic.ext.StorageService").await {
        Ok(_) => Ok(false),  // We got the name, no other instance
        Err(e) if e.to_string().contains("already owned") => Ok(true),
        Err(e) => Err(e.into()),
    }
}
```

### Files Affected
- `storage-service/src/main.rs`

---

## GAP-015: No Integration Test Coverage for D-Bus Interfaces

### Decision
Create integration test scaffolding with serialization tests.

### Test Categories

1. **Serialization Round-trips**: Verify JSON serialization matches expected schema
2. **Error Mapping**: Verify D-Bus errors map to correct ClientError variants
3. **Auth Checks**: Verify Polkit integration (may require mock)

### Files to Create
- `tests/integration/serialization.rs`
- `tests/integration/client_errors.rs`

---

## GAP-016: Unclear Atomic Update Strategy in UiDrive

### Decision
Document refresh strategy and add RefreshResult enum.

### Documentation

```rust
/// # Refresh Strategy
///
/// - **Full refresh** (`refresh()`): Called on startup, after partition create/delete,
///   or when device events fire
/// - **Atomic refresh** (`refresh_volume()`): Called after mount/unmount/format of
///   existing volume
/// - **Not atomic-safe**: Partition creation/deletion (always triggers full refresh)
///
/// Atomic refresh is a performance optimization but NOT a correctness guarantee.
/// If atomic refresh fails or returns NotFound, caller should schedule a full refresh.
```

### Files Affected
- `storage-ui/src/models/ui_drive.rs`

---

## Summary of Decisions

| Gap | Decision | Effort |
|-----|----------|--------|
| GAP-002 | Required Connection | Medium |
| GAP-003 | Arc-wrap clients (with GAP-006) | Medium |
| GAP-004 | Parent path in flatten | Low |
| GAP-005 | Replace unwrap with error propagation | High |
| GAP-006 | ClientPool pattern | Medium |
| GAP-007 | Add serialization tests | Low |
| GAP-008 | Input validation | Medium |
| GAP-009 | Delete conversions.rs | Low |
| GAP-010 | Link TODOs to issues | Low |
| GAP-011 | Handle mutex poisoning | Low |
| GAP-012 | Preserve error context | Low |
| GAP-013 | Timeouts + progress | Medium |
| GAP-014 | Startup check | Low |
| GAP-015 | Integration test scaffolding | Medium |
| GAP-016 | Document refresh strategy | Low |
