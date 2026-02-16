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

---

## APPENDIX B: Polkit Authorization & User Context Research

*Added during implementation: Security audit revealed critical vulnerabilities in authorization checking.*

### 7. Procedural Macro for Authorized Interface

**Question**: How can we implement a `#[authorized_interface()]` macro that wraps `#[zbus::interface]` and adds Polkit authorization?

#### Decision: Create a procedural macro crate `storage-service-macros`

**Rationale**:
- Procedural macros require a separate crate type (`proc-macro = true`)
- Macro can generate the `#[zbus::interface]` code with authorization wrapper
- Centralizes authorization logic in one place
- Eliminates boilerplate in each interface method

**Macro Design**:

```rust
// storage-service-macros/src/lib.rs

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_parse, ItemImpl};

#[proc_macro_attribute]
pub fn authorized_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let action_id = parse_macro_parse!(attr as LitStr);
    let impl_block = parse_macro_parse!(item as ItemImpl);

    // Generate code that:
    // 1. Extracts header and connection from zbus
    // 2. Gets caller info from header.sender()
    // 3. Checks Polkit authorization
    // 4. Injects CallerInfo parameter into method body

    let expanded = quote! {
        #[zbus::interface(name = "org.cosmic.ext.StorageService")]
        impl #impl_block {
            // Generated authorization wrapper for each method
        }
    };

    expanded.into()
}
```

**Usage Pattern**:

```rust
// In service methods
#[authorized_interface(action = "org.cosmic.ext.storage-service.mount")]
async fn mount(
    &self,
    caller: CallerInfo,  // Auto-injected by macro
    device: String,
    mount_point: String,
    options: MountOptions,
) -> zbus::fdo::Result<String> {
    // caller.uid and caller.username available
    // Authorization already verified
    crate::filesystem::mount_filesystem(&device, &mount_point, options, Some(caller.uid)).await
}
```

**Alternatives Considered**:

| Approach | Rejected Because |
|----------|------------------|
| Manual `#[zbus(header)]` on each method | Boilerplate across 60+ methods, error-prone |
| Runtime middleware/dispatch | zbus doesn't support middleware hooks |
| Base trait with default auth method | Still requires per-method invocation |
| Wrapper struct around impl blocks | Doesn't compose with `#[interface]` macro |

---

### 8. Extracting Caller Identity from D-Bus Messages

**Question**: How do we correctly identify the calling user in a D-Bus method?

#### Decision: Use `#[zbus(header)]` parameter with `header.sender()` to get unique bus name, then look up UID

**Rationale**:
- `MessageHeader::sender()` returns the caller's unique bus name (e.g., `:1.42`)
- `DBusProxy::get_connection_unix_user()` converts bus name to UID
- This is the CORRECT approach, unlike `connection.unique_name()` which returns the service's own name

**Implementation Pattern**:

```rust
use zbus::message::Header as MessageHeader;

async fn method(
    #[zbus(header)] header: MessageHeader<'_>,
    #[zbus(connection)] connection: &Connection,
) -> zbus::fdo::Result<()> {
    // Get the ACTUAL caller's bus name
    let sender = header.sender()
        .ok_or_else(|| zbus::fdo::Error::Failed("No sender".into()))?;

    // Look up the caller's UID
    let dbus_proxy = zbus::fdo::DBusProxy::new(connection).await?;
    let uid = dbus_proxy.get_connection_unix_user(sender.as_ref()).await?;

    // Now we have the real caller's UID
}
```

**Common Mistake (Current Bug)**:

```rust
// WRONG - returns the service's own bus name!
let sender = connection.unique_name().unwrap().to_string();
// This checks if root is authorized (always yes)
```

---

### 9. Polkit Authorization with Correct Subject

**Question**: How do we construct the Polkit Subject for authorization checking?

#### Decision: Use `Subject::new_for_owner(pid, None, None)` with PID from D-Bus

**Rationale**:
- Polkit needs a Subject to check authorization against
- `new_for_owner()` takes a PID and creates the correct subject type
- We get PID from `get_connection_unix_process_id()` using the caller's bus name

**Implementation Pattern**:

```rust
use zbus_polkit::policykit1::{AuthorityProxy, Subject, CheckAuthorizationFlags};

async fn check_authorization(
    connection: &Connection,
    sender: &str,  // Caller's unique bus name from header.sender()
    action_id: &str,
) -> Result<bool, ServiceError> {
    let authority = AuthorityProxy::new(connection).await?;

    // Get caller's PID from their bus name
    let dbus_proxy = zbus::fdo::DBusProxy::new(connection).await?;
    let bus_name: zbus::names::BusName = sender.try_into()?;
    let pid = dbus_proxy.get_connection_unix_process_id(bus_name).await?;

    // Create subject for the ACTUAL caller
    let subject = Subject::new_for_owner(pid, None, None)?;

    let result = authority.check_authorization(
        &subject,
        action_id,
        &HashMap::new(),
        CheckAuthorizationFlags::AllowUserInteraction.into(),
        "",
    ).await?;

    Ok(result.is_authorized)
}
```

---

### 10. UDisks2 User Context Passthrough

**Question**: Which UDisks2 operations need user context, and how do we pass it?

#### Decision: Use `as-user` option for mount path, `uid` option for file ownership

**Rationale**:
- UDisks2's `Filesystem.Mount()` supports an `as-user` option that creates mount points under `/run/media/<username>/`
- The `uid` mount option ensures files on FAT/NTFS/exFAT are owned by the user
- Both options together provide complete user context

**UDisks2 Operations Requiring User Context**:

| Operation | Option | Effect |
|-----------|--------|--------|
| `Filesystem.Mount()` | `as-user=<username>` | Mount point created at `/run/media/<username>/` |
| `Filesystem.Mount()` | `uid=<uid>` | Files owned by user (FAT/NTFS/exFAT) |
| `Filesystem.TakeOwnership()` | Run as user | Files chown'd to user |

**Implementation Pattern**:

```rust
// storage-dbus/src/filesystem/mount.rs

pub async fn mount_filesystem(
    device_path: &str,
    options: MountOptions,
    caller_uid: Option<u32>,  // NEW parameter
) -> Result<String, DiskError> {
    let mut opts: HashMap<&str, Value<'_>> = HashMap::new();

    if let Some(uid) = caller_uid {
        // Get username from UID
        if let Some(username) = get_username_from_uid(uid) {
            // This sets mount path to /run/media/<username>/
            opts.insert("as-user", Value::from(username));
            // This sets file ownership on FAT/NTFS
            opts.insert("uid", Value::from(uid));
        }
    }

    // ... rest of mount logic
}

fn get_username_from_uid(uid: u32) -> Option<String> {
    let pw = unsafe { libc::getpwuid(uid) };
    if pw.is_null() { return None; }
    unsafe {
        std::ffi::CStr::from_ptr((*pw).pw_name)
            .to_str().ok().map(|s| s.to_string())
    }
}
```

---

### 11. Methods Requiring Authorization Fix

**Question**: Which service methods currently have broken authorization?

#### Identified Methods

All methods using `check_polkit_auth()` or `require_authorization()` without explicitly passing the sender from `header.sender()` are affected.

**Affected Files**:

| File | Methods | Impact |
|------|---------|--------|
| `storage-service/src/filesystems.rs` | `mount`, `unmount`, `format`, `set_label`, `take_ownership` | HIGH - All filesystem ops bypassed |
| `storage-service/src/partitions.rs` | `create`, `delete`, `resize`, `set_type`, `set_flags` | HIGH - All partition ops bypassed |
| `storage-service/src/luks.rs` | `unlock`, `lock`, `format_luks`, `change_passphrase` | HIGH - All encryption ops bypassed |
| `storage-service/src/btrfs.rs` | `create_subvolume`, `delete_subvolume`, `create_snapshot` | MEDIUM - Btrfs ops bypassed |
| `storage-service/src/zram.rs` | `create_zram_device`, `destroy_zram_device` | MEDIUM - Zram ops bypassed |
| `storage-service/src/disks.rs` | `eject`, `power_off` | MEDIUM - Disk control bypassed |

**Total**: ~60+ method calls across 7 files

---

## Final Summary of Decisions

| Topic | Decision | Scope |
|-------|----------|-------|
| UI Connection Sharing | `tokio::sync::OnceCell` with static getter | storage-ui |
| DBus Library Connection | `Arc<Connection>` stored in `DiskManager` | storage-dbus |
| Protected Paths | Static constant with canonicalization + prefix matching | storage-service |
| FSTools Detection | Enhance existing detection, add D-Bus method | storage-service |
| Error Communication | Use existing `UnmountResult.error` field | storage-service |
| **Authorization Macro** | **Create `storage-service-macros` crate with `#[authorized_interface()]`** | **NEW CRATE** |
| **Caller Identity** | **Use `header.sender()` + `get_connection_unix_user()`** | **storage-service** |
| **UDisks2 User Passthrough** | **`as-user` + `uid` options for mount operations** | **storage-dbus** |

## Revised Implementation Priority

1. **Authorization Macro** (CRITICAL SECURITY): Fixes complete Polkit bypass
2. **User Context Passthrough** (CRITICAL USABILITY): Mounts accessible to users
3. **Layer 2** (storage-dbus → UDisks2): Performance
4. **Layer 1** (storage-ui → storage-service): Performance
5. **System Path Protection**: Safety feature
6. **FSTools Consolidation**: Maintainability
