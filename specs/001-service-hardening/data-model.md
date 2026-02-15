# Data Model: Service Hardening

**Feature**: 001-service-hardening
**Date**: 2026-02-15

## New Types

### FilesystemToolInfo

Describes a filesystem formatting tool and its availability.

```rust
/// Information about a filesystem formatting tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemToolInfo {
    /// Filesystem type identifier (e.g., "ext4", "btrfs")
    pub fs_type: String,

    /// Human-readable filesystem name (e.g., "EXT4", "Btrfs")
    pub fs_name: String,

    /// Command to check for availability (e.g., "mkfs.ext4")
    pub command: String,

    /// Package name hint for installation (e.g., "e2fsprogs")
    pub package_hint: String,

    /// Whether the tool is currently available on this system
    pub available: bool,
}
```

**Validation Rules**:
- `fs_type` must be non-empty and lowercase
- `command` must be non-empty
- `available` is determined by `which::which(command).is_ok()`

**State Transitions**: None (immutable after creation)

---

## Modified Types

### UnmountResult (existing)

No structural changes. The `error` field will contain new error message for protected paths.

```rust
// Existing - no changes
pub struct UnmountResult {
    pub success: bool,
    pub error: Option<String>,
    pub blocking_processes: Vec<ProcessInfo>,
}
```

**New Error Case**:
- `success: false`
- `error: Some("Cannot kill processes on system path '...'")`
- `blocking_processes: Vec::new()`

---

## Internal Types (not exposed via D-Bus)

### SharedConnection (storage-ui)

Internal singleton for D-Bus connection caching.

```rust
// Internal to storage-ui/src/client/connection.rs
// Not exposed publicly, only used within client module
```

**Lifecycle**:
1. First call to `shared_connection()` establishes connection
2. Connection cached in static `OnceLock`
3. Subsequent calls return cached reference
4. Connection lives for application lifetime

---

### ProtectedPath (storage-service)

Internal module for system path protection logic.

```rust
// Internal to storage-service/src/protected_paths.rs

/// List of system paths protected from kill_processes unmount
pub const PROTECTED_SYSTEM_PATHS: &[&str] = &[/* ... */];

/// Check if a mount point is a protected system path
pub fn is_protected_path(mount_point: &str) -> bool;

/// Get canonical path, handling errors gracefully
fn canonicalize_path(path: &str) -> PathBuf;
```

**Validation Rules**:
- Uses `std::fs::canonicalize` for symlink resolution
- Falls back to original path if canonicalization fails
- Matches exact path or any subdirectory

---

## Relationships

```
┌─────────────────────────────────────────────────────────────┐
│                     storage-ui (GUI)                        │
│                                                             │
│  SharedConnection ──────────────────────────────────────────┤
│       │                                                     │
│       └─────> DisksClient, FilesystemsClient, etc.          │
│                   │                                         │
│                   └─────> D-Bus calls to service            │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ D-Bus (system bus)
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  storage-service (D-Bus Service)            │
│                                                             │
│  FilesystemsHandler                                         │
│       │                                                     │
│       ├──> supported_tools: Vec<String>                     │
│       │    (existing, used for format validation)           │
│       │                                                     │
│       └──> filesystem_tools: Vec<FilesystemToolInfo>        │
│            (new, exposed via get_filesystem_tools())        │
│                                                             │
│  ProtectedPath (module)                                     │
│       │                                                     │
│       └──> PROTECTED_SYSTEM_PATHS: &[&str]                  │
│            is_protected_path() -> bool                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Data Flow

### Connection Sharing Flow

```
UI Action
    │
    ▼
Client::new()
    │
    ▼
shared_connection() ───> OnceLock::get_or_init()
    │                          │
    │                          ▼ (first time only)
    │                    Connection::system()
    │                          │
    │                          ▼
    │                    Cache in static
    │
    ▼
Create proxy from shared connection
```

### Protected Path Check Flow

```
unmount(device, force=true, kill_processes=true)
    │
    ▼
Get mount point from device
    │
    ▼
is_protected_path(mount_point)?
    │
    ├─── YES ──> Return UnmountResult {
    │                success: false,
    │                error: Some("Cannot kill processes..."),
    │                blocking_processes: []
    │              }
    │
    └─── NO ───> Proceed with normal unmount logic
```

### FSTools Query Flow

```
UI needs filesystem options
    │
    ▼
FilesystemsClient::get_filesystem_tools()
    │
    ▼
D-Bus call to service
    │
    ▼
FilesystemsHandler::get_filesystem_tools()
    │
    ▼
Return JSON Vec<FilesystemToolInfo>
    │
    ▼
UI filters available types only
```

---

## APPENDIX: Layer 2 - Storage-DBus Connection Model

*Added during planning phase: Connection caching for storage-dbus → UDisks2 layer.*

### DiskManager (storage-dbus) - Modified

The existing `DiskManager` struct will be enhanced to cache and expose its D-Bus connection.

```rust
// storage-dbus/src/disk/manager.rs

/// Manages disk discovery and change detection
pub struct DiskManager {
    /// Cached D-Bus connection to system bus (for UDisks2)
    connection: Arc<Connection>,

    /// Existing fields...
    drives: HashMap<String, DriveInfo>,
    volumes: HashMap<String, VolumeInfo>,
    // ...
}

impl DiskManager {
    /// Create a new DiskManager with cached D-Bus connection
    pub async fn new() -> Result<Self, DiskError> {
        let connection = Arc::new(
            Connection::system()
                .await
                .map_err(|e| DiskError::Connection(e.to_string()))?
        );

        Ok(Self {
            connection,
            drives: HashMap::new(),
            volumes: HashMap::new(),
            // ...
        })
    }

    /// Get a reference to the cached connection for reuse
    pub fn connection(&self) -> &Arc<Connection> {
        &self.connection
    }
}
```

**Lifecycle**:
1. Created once when storage-service starts
2. Connection established eagerly in constructor
3. Lives for service lifetime
4. Shared across all discovery operations

---

### Discovery Functions (storage-dbus) - Modified

Discovery functions will accept the manager reference to reuse its connection.

```rust
// storage-dbus/src/disk/discovery.rs

/// Discover all disks with their volumes using cached connection
pub async fn get_disks_with_volumes(
    manager: &DiskManager
) -> Result<Vec<(DiskInfo, Vec<VolumeInfo>)>, DiskError> {
    let connection = manager.connection();  // REUSE cached connection

    // Create proxies from the shared connection
    let manager_proxy = UDisks2ManagerProxy::new(connection.as_ref()).await?;
    let block_objects = manager_proxy.get_block_objects().await?;

    // ... rest of discovery logic unchanged
}
```

**Key Change**: Function signature changed from `()` to `(&DiskManager)` parameter.

---

## Updated Relationships Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     storage-ui (GUI)                        │
│                                                             │
│  SharedConnection (OnceLock<Connection>)                    │
│       │                                                     │
│       └─────> DisksClient, FilesystemsClient, etc.          │
└─────────────────────────┬───────────────────────────────────┘
                          │ D-Bus (REUSED connection)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  storage-service (D-Bus Service)            │
│                                                             │
│  DiskManager                                                │
│       │                                                     │
│       └──> connection: Arc<Connection> ─────────────────────┤
│                                                              │
│  Discovery calls use manager.connection()                   │
└─────────────────────────┬───────────────────────────────────┘
                          │ library calls
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  storage-dbus (Library)                     │
│                                                             │
│  get_disks_with_volumes(manager: &DiskManager)              │
│       │                                                     │
│       └──> Uses manager.connection() (Arc<Connection>)      │
│                                                             │
│  Proxies created from cached connection:                    │
│       - UDisks2ManagerProxy                                 │
│       - BlockProxy, DriveProxy, etc.                        │
└─────────────────────────┬───────────────────────────────────┘
                          │ D-Bus (REUSED connection)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    UDisks2 (System Service)                 │
└─────────────────────────────────────────────────────────────┘
```

---

## Data Flow - Layer 2 Connection Reuse

```
Service Request (e.g., list_disks)
    │
    ▼
DisksHandler::list_disks()
    │
    ▼
get_disks_with_volumes(&self.manager)  ───> Pass manager reference
    │
    ▼
manager.connection()  ───> Get Arc<Connection>
    │
    ▼
UDisks2ManagerProxy::new(connection.as_ref())
    │
    ▼
Enumerate disks via UDisks2 (using existing connection)
```

**Performance Impact**: No new D-Bus connection created for discovery operations after initial service startup.
