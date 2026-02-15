# Research: Service Hardening

**Feature**: 001-service-hardening
**Date**: 2026-02-15
**Status**: Complete

## Research Topics

### 1. Persistent D-Bus Connection Patterns in Rust/zbus

**Question**: What is the best pattern for sharing a single D-Bus connection across multiple async client instances?

#### Decision: Use `tokio::sync::OnceCell` with a static getter function

**Rationale**:
- `tokio::sync::OnceCell` provides async-safe lazy initialization
- Static lifetime ensures connection persists for application duration
- zbus `Connection` is `Clone` and `Send`, making it safe to share
- Each proxy can be created from the same cloned connection reference

**Alternatives Considered**:

| Pattern | Rejected Because |
|---------|------------------|
| `std::sync::LazyLock` | Requires blocking on async connection establishment |
| Pass connection through all call sites | Invasive changes to existing code |
| Global `Mutex<Option<Connection>>` | More complex, same result as OnceCell |
| Create connection in `AppModel` and store in state | Requires threading through entire app |

**Implementation Pattern**:

```rust
use std::sync::OnceLock;
use zbus::Connection;

static SYSTEM_CONNECTION: OnceLock<Connection> = OnceLock::new();

pub async fn shared_connection() -> Result<&'static Connection, ClientError> {
    if let Some(conn) = SYSTEM_CONNECTION.get() {
        return Ok(conn);
    }

    // Race condition is acceptable - multiple connections during startup is fine
    let conn = Connection::system()
        .await
        .map_err(|e| ClientError::Connection(e.to_string()))?;

    // Ignore error if already set (another task won the race)
    let _ = SYSTEM_CONNECTION.set(conn);

    Ok(SYSTEM_CONNECTION.get().unwrap())
}
```

**References**:
- zbus documentation on connection sharing
- tokio::sync::OnceCell vs std::sync::OnceLock for async init

---

### 2. Protected System Path Detection

**Question**: How should protected paths be defined and matched against mount points?

#### Decision: Static constant array with prefix matching using path canonicalization

**Rationale**:
- Static list is simple and covers all critical paths
- Prefix matching handles subdirectories (e.g., `/boot/efi`)
- Path canonicalization handles symlinks (e.g., `/home` -> `/mnt/home`)
- Service-side validation ensures security regardless of UI behavior

**Protected Paths**:

```rust
pub const PROTECTED_SYSTEM_PATHS: &[&str] = &[
    "/",        // Root filesystem
    "/boot",    // Bootloader
    "/boot/efi", // EFI system partition
    "/home",    // User data
    "/usr",     // System programs
    "/var",     // Variable data (logs, databases)
    "/etc",     // System configuration
    "/opt",     // Optional software
    "/srv",     // Service data
    "/tmp",     // Temporary files (may have important running processes)
];
```

**Matching Logic**:

```rust
pub fn is_protected_path(mount_point: &str) -> bool {
    // Canonicalize to resolve symlinks
    let canonical = std::fs::canonicalize(mount_point)
        .unwrap_or_else(|_| PathBuf::from(mount_point));

    let canonical_str = canonical.to_string_lossy();

    PROTECTED_SYSTEM_PATHS.iter().any(|protected| {
        // Exact match or subdirectory
        canonical_str == *protected ||
        canonical_str.starts_with(&format!("{}/", protected))
    })
}
```

**Alternatives Considered**:

| Approach | Rejected Because |
|----------|------------------|
| Dynamic detection from /proc/mounts | Unnecessarily complex for static set |
| Configure via config file | Security-critical paths should be hardcoded |
| Check mount flags (e.g., shared) | Not all critical mounts have distinguishing flags |

---

### 3. Filesystem Tool Detection Architecture

**Question**: How should filesystem tool detection be structured in the service?

#### Decision: Enhance existing `FilesystemsHandler::detect_filesystem_tools()` with comprehensive tool list, expose via D-Bus property

**Rationale**:
- Detection already exists in `FilesystemsHandler::new()`
- `supported_features` property already exists on `StorageService`
- Can extend with granular filesystem tool info via new method
- Maintains backwards compatibility

**Enhanced Detection**:

```rust
// In storage-service/src/filesystems.rs
fn detect_filesystem_tools() -> Vec<FilesystemToolInfo> {
    let tools = vec![
        ("ext4", "mkfs.ext4", "e2fsprogs"),
        ("xfs", "mkfs.xfs", "xfsprogs"),
        ("btrfs", "mkfs.btrfs", "btrfs-progs"),
        ("vfat", "mkfs.vfat", "dosfstools"),
        ("ntfs", "mkfs.ntfs", "ntfs-3g"),
        ("exfat", "mkfs.exfat", "exfatprogs"),
        ("f2fs", "mkfs.f2fs", "f2fs-tools"),
        ("udf", "mkudffs", "udftools"),
    ];

    tools.into_iter()
        .map(|(fs_type, command, package)| FilesystemToolInfo {
            fs_type: fs_type.to_string(),
            command: command.to_string(),
            package_hint: package.to_string(),
            available: which::which(command).is_ok(),
        })
        .collect()
}
```

**D-Bus Exposure Options**:

| Option | Pros | Cons |
|--------|------|------|
| Extend `supported_features` | Simple, already exists | Less structured, string parsing needed |
| New `get_filesystem_tools()` method | Structured JSON response | New method to maintain |
| Property on Filesystems interface | Cached, queryable | More complex implementation |

**Decision**: Add new `get_filesystem_tools()` method returning JSON array for structured data, while keeping `supported_features` as simple string list for backward compatibility.

---

### 4. Error Handling for Protected Paths

**Question**: How should protected path errors be communicated to the UI?

#### Decision: Use structured error in `UnmountResult` with specific error type

**Rationale**:
- `UnmountResult` already has `error: Option<String>` field
- UI already handles displaying errors from this field
- No D-Bus contract changes needed

**Error Message Format**:

```rust
// In service
if kill_processes && is_protected_path(&mount_point) {
    let result = UnmountResult {
        success: false,
        error: Some(format!(
            "Cannot kill processes on system path '{}'. \
             This filesystem is critical for system operation.",
            mount_point
        )),
        blocking_processes: Vec::new(),
    };
    return Ok(serde_json::to_string(&result)?);
}
```

**UI Handling**:
- Error appears in existing error display mechanism
- User sees clear message explaining why operation was rejected
- No code changes needed in UI error handling path

---

## Summary of Decisions

| Topic | Decision |
|-------|----------|
| Connection Sharing | `tokio::sync::OnceCell` with static getter |
| Protected Paths | Static constant with canonicalization + prefix matching |
| FSTools Detection | Enhance existing detection, add D-Bus method for structured data |
| Error Communication | Use existing `UnmountResult.error` field |

## No Outstanding Clarifications

All technical decisions have been made. Implementation can proceed.

---

## APPENDIX: Layer 2 - Storage-DBus → UDisks2 Connection Research

*Added during planning phase: Investigation revealed critical performance bottleneck in storage-dbus library.*

### 5. Storage-DBus Connection Caching

**Question**: How should `storage-dbus` cache its UDisks2 connection for reuse across multiple discovery operations?

#### Current Problem

The `get_disks_with_volumes_inner()` function in `storage-dbus/src/disk/discovery.rs:417` creates a new `Connection::system()` on every call:

```rust
async fn get_disks_with_volumes_inner() -> Result<Vec<(DiskInfo, Vec<VolumeInfo>)>> {
    let connection = Connection::system().await?;  // FRESH CONNECTION EACH CALL
    // This function is called 9+ times in storage-service/src/disks.rs
}
```

#### Decision: Store `Arc<Connection>` in `DiskManager` and pass to discovery functions

**Rationale**:
- `DiskManager` already exists as a stateful struct in storage-dbus
- `DiskManager` has a lifecycle tied to the service (created once at service start)
- `Arc<Connection>` allows shared ownership across functions
- Minimal API changes - just add connection parameter to discovery functions

**Implementation Pattern**:

```rust
// storage-dbus/src/disk/manager.rs
pub struct DiskManager {
    connection: Arc<Connection>,
    // existing fields...
}

impl DiskManager {
    pub async fn new() -> Result<Self, DiskError> {
        let connection = Arc::new(Connection::system().await?);
        Ok(Self {
            connection,
            // ...
        })
    }

    /// Get a reference to the cached connection for reuse
    pub fn connection(&self) -> &Arc<Connection> {
        &self.connection
    }
}

// storage-dbus/src/disk/discovery.rs
pub async fn get_disks_with_volumes(
    manager: &DiskManager  // Now takes manager reference
) -> Result<Vec<(DiskInfo, Vec<VolumeInfo>)>> {
    let connection = manager.connection();  // REUSE cached connection
    // ... rest of function unchanged, just use `connection`
}
```

**Call Site Updates**:

All call sites in `storage-service/src/disks.rs` that call `get_disks_with_volumes()` need to pass the manager:

```rust
// Before
let disks = get_disks_with_volumes().await?;

// After
let disks = get_disks_with_volumes(&self.manager).await?;
```

**Alternatives Considered**:

| Approach | Rejected Because |
|----------|------------------|
| Static OnceCell in storage-dbus | DiskManager already provides natural lifecycle |
| Thread-local connection | More complex, async compatibility issues |
| Create connection per-call (current) | Causes the performance problems we're fixing |

---

### 6. Connection Lifetime in Service Context

**Question**: When should the DiskManager connection be established?

#### Decision: Eager initialization when DiskManager is created

**Rationale**:
- DiskManager is created once at service startup
- Early initialization catches connection problems immediately
- First disk enumeration will be fast (no lazy init delay)
- Connection lifetime matches service lifetime

**Sequence**:

```
Service Start
    │
    ▼
DiskManager::new()
    │
    ▼
Connection::system()  ← ONE connection established here
    │
    ▼
All subsequent operations reuse this connection
```

---

## Updated Summary of Decisions

| Topic | Decision | Scope |
|-------|----------|-------|
| UI Connection Sharing | `tokio::sync::OnceCell` with static getter | storage-ui |
| DBus Library Connection | `Arc<Connection>` stored in `DiskManager` | storage-dbus |
| Protected Paths | Static constant with canonicalization + prefix matching | storage-service |
| FSTools Detection | Enhance existing detection, add D-Bus method | storage-service |
| Error Communication | Use existing `UnmountResult.error` field | storage-service |

## Implementation Priority

1. **Layer 2** (storage-dbus → UDisks2): Higher impact - affects all service operations
2. **Layer 1** (storage-ui → storage-service): Important but less frequent calls
3. **System Path Protection**: Safety feature, independent of performance
4. **FSTools Consolidation**: Lower priority, maintainability improvement
