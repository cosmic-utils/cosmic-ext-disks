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

---

## APPENDIX B: Authorization & User Context Types

*Added during implementation: Security audit revealed need for caller identity tracking.*

### CallerInfo (storage-common)

Information about the D-Bus caller, extracted from message header and provided to service methods.

```rust
/// Information about a D-Bus method caller
#[derive(Debug, Clone)]
pub struct CallerInfo {
    /// Unix user ID of the calling process
    pub uid: u32,

    /// Username resolved from UID (via getpwuid)
    /// May be None if user lookup fails
    pub username: Option<String>,

    /// D-Bus unique bus name of the caller (e.g., ":1.42")
    pub sender: String,
}
```

**Validation Rules**:
- `uid` is always present (from D-Bus)
- `username` is resolved via `libc::getpwuid`, may be `None`
- `sender` is the unique bus name from message header

**Lifecycle**:
1. Extracted from D-Bus message header by `#[authorized_interface]` macro
2. Populated with UID via `get_connection_unix_user()`
3. Username resolved via `getpwuid`
4. Passed to method body for use in operations

---

### AuthorizedInterface Attribute (storage-service-macros)

Procedural macro attribute that wraps `#[zbus::interface]` with Polkit authorization.

**Attribute Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `action` | `&str` | Yes | Polkit action ID to check |

**Generated Code Behavior**:

1. Adds `#[zbus(header)]` and `#[zbus(connection)]` parameters
2. Extracts sender from `header.sender()`
3. Looks up caller UID via `get_connection_unix_user()`
4. Resolves username via `getpwuid`
5. Checks Polkit authorization with correct subject
6. If authorized: creates `CallerInfo` and calls method body
7. If not authorized: returns `zbus::fdo::Error::AccessDenied`

**Usage**:

```rust
#[authorized_interface(action = "org.cosmic.ext.storage-service.mount")]
async fn mount(
    &self,
    caller: CallerInfo,  // Auto-injected
    device: String,
    mount_point: String,
    options_json: String,
) -> zbus::fdo::Result<String> {
    // Authorization already verified
    // Use caller.uid for UDisks2 passthrough
}
```

---

### Modified Types - User Context Passthrough

#### MountOptions (existing - extended)

The mount function signature is extended to accept optional caller UID.

```rust
// storage-dbus/src/filesystem/mount.rs

pub async fn mount_filesystem(
    device_path: &str,
    _mount_point: &str,
    options: MountOptions,
    caller_uid: Option<u32>,  // NEW: for UDisks2 user context
) -> Result<String, DiskError>
```

**Behavior with caller_uid**:
- If `Some(uid)`: Pass `as-user=<username>` and `uid=<uid>` to UDisks2
- If `None`: Use default UDisks2 behavior (mount as root)

---

## Updated Relationships Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     storage-ui (GUI)                        │
│                                                             │
│  SharedConnection (OnceLock<Connection>)                    │
│       │                                                     │
│       └─────> D-Bus method call with sender identity        │
└─────────────────────────┬───────────────────────────────────┘
                          │ D-Bus (caller's unique name in header)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  storage-service (D-Bus Service)            │
│                                                             │
│  #[authorized_interface(action = "...")]                    │
│       │                                                     │
│       ├──> 1. Extract sender from MessageHeader             │
│       ├──> 2. Look up UID via get_connection_unix_user()    │
│       ├──> 3. Resolve username via getpwuid                 │
│       ├──> 4. Check Polkit authorization                    │
│       │                                                     │
│       └──> If authorized: Create CallerInfo, call method    │
│                                                             │
│  Method receives: caller: CallerInfo                        │
│       │                                                     │
│       └─────> Pass caller.uid to storage-dbus operations    │
└─────────────────────────┬───────────────────────────────────┘
                          │ library calls with caller context
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  storage-dbus (Library)                     │
│                                                             │
│  mount_filesystem(device, mount_point, options, caller_uid) │
│       │                                                     │
│       └─────> If caller_uid:                                │
│                   - Resolve username from UID               │
│                   - Pass as-user=<username> to UDisks2      │
│                   - Pass uid=<uid> to UDisks2               │
└─────────────────────────┬───────────────────────────────────┘
                          │ D-Bus (as-user option in call)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    UDisks2 (System Service)                 │
│                                                             │
│  Mount created at: /run/media/<username>/                   │
│  Files owned by: <uid>                                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Data Flow - Authorization & User Context

### Authorization Flow

```
D-Bus Method Call
    │
    ├──> Message Header contains sender (":1.42")
    │
    ▼
#[authorized_interface] Macro
    │
    ├──> 1. header.sender() → ":1.42"
    │
    ├──> 2. get_connection_unix_user(":1.42") → uid=1000
    │
    ├──> 3. getpwuid(1000) → username="alice"
    │
    ├──> 4. get_connection_unix_process_id(":1.42") → pid=12345
    │
    ├──> 5. Subject::new_for_owner(12345, None, None)
    │
    ├──> 6. Polkit check_authorization(subject, action_id)
    │         │
    │         ├──> Authorized → Continue
    │         └──> Not Authorized → Return AccessDenied
    │
    ▼
Method Body
    │
    └──> Receives CallerInfo { uid: 1000, username: Some("alice"), sender: ":1.42" }
```

### Mount with User Context Flow

```
mount(caller: CallerInfo, device, ...)
    │
    ▼
mount_filesystem(device, options, Some(caller.uid))
    │
    ▼
get_username_from_uid(1000) → "alice"
    │
    ▼
UDisks2 Filesystem.Mount({
    "as-user": "alice",      // Mount path: /run/media/alice/
    "uid": 1000,             // File ownership: alice
    "options": "rw,nosuid"   // Standard mount options
})
    │
    ▼
Mount point: /run/media/alice/USB_DRIVE
Files owned by: alice (uid 1000)
```

---

## Migration Checklist

### Types to Create

- [ ] `storage-common/src/caller.rs` - `CallerInfo` struct
- [ ] `storage-service-macros/Cargo.toml` - proc-macro crate config
- [ ] `storage-service-macros/src/lib.rs` - `#[authorized_interface]` macro

### Functions to Modify

| Function | Location | Change |
|----------|----------|--------|
| `mount_filesystem()` | `storage-dbus/src/filesystem/mount.rs` | Add `caller_uid: Option<u32>` param |
| All interface methods | `storage-service/src/*.rs` | Migrate to `#[authorized_interface]` |

### Functions to Deprecate

| Function | Location | Replacement |
|----------|----------|-------------|
| `check_polkit_auth()` | `storage-service/src/auth.rs` | `#[authorized_interface]` macro |
