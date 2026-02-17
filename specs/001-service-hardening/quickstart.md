# Quickstart: Service Hardening Implementation

**Feature**: 001-service-hardening
**Date**: 2026-02-15

## Prerequisites

- Rust stable toolchain
- Running UDisks2 service
- Root access for testing storage-service

## Implementation Order

The three phases can be implemented independently, but recommended order:

1. **Phase 1: Shared Connection** (improves all subsequent development)
2. **Phase 2: Protected Paths** (safety-critical)
3. **Phase 3: FSTools Consolidation** (benefits from Phase 1)

---

## Phase 1: Shared Connection Manager

### Step 1: Create connection module

Create `storage-ui/src/client/connection.rs`:

```rust
// SPDX-License-Identifier: GPL-3.0-only

//! Shared D-Bus connection management

use std::sync::OnceLock;
use zbus::Connection;
use crate::client::error::ClientError;

static SYSTEM_CONNECTION: OnceLock<Connection> = OnceLock::new();

/// Get or create the shared system bus connection
pub async fn shared_connection() -> Result<&'static Connection, ClientError> {
    if let Some(conn) = SYSTEM_CONNECTION.get() {
        return Ok(conn);
    }

    let conn = Connection::system()
        .await
        .map_err(|e| ClientError::Connection(e.to_string()))?;

    let _ = SYSTEM_CONNECTION.set(conn);
    Ok(SYSTEM_CONNECTION.get().unwrap())
}
```

### Step 2: Update client modules

In each client (`disks.rs`, `filesystems.rs`, etc.), replace:

```rust
// Before
pub async fn new() -> Result<Self, ClientError> {
    let conn = Connection::system().await?;
    let proxy = Proxy::new(&conn).await?;
    Ok(Self { proxy })
}
```

With:

```rust
// After
use super::connection::shared_connection;

pub async fn new() -> Result<Self, ClientError> {
    let conn = shared_connection().await?;
    let proxy = Proxy::new(conn).await?;
    Ok(Self { proxy })
}
```

### Step 3: Export module

In `storage-ui/src/client/mod.rs`, add:

```rust
mod connection;
```

### Verification

```bash
# Build
cargo build --package storage-ui

# Test startup time improvement
time ./target/debug/storage-ui  # Should be noticeably faster
```

---

## Phase 2: Protected System Paths

### Step 1: Create protected_paths module

Create `storage-service/src/protected_paths.rs`:

```rust
// SPDX-License-Identifier: GPL-3.0-only

//! Protected system path definitions and validation

use std::path::PathBuf;

/// System paths protected from kill_processes during unmount
pub const PROTECTED_SYSTEM_PATHS: &[&str] = &[
    "/",
    "/boot",
    "/boot/efi",
    "/home",
    "/usr",
    "/var",
    "/etc",
    "/opt",
    "/srv",
    "/tmp",
];

/// Check if a mount point is a protected system path
pub fn is_protected_path(mount_point: &str) -> bool {
    let canonical = canonicalize_path(mount_point);
    let canonical_str = canonical.to_string_lossy();

    PROTECTED_SYSTEM_PATHS.iter().any(|protected| {
        canonical_str == *protected || canonical_str.starts_with(&format!("{}/", protected))
    })
}

fn canonicalize_path(path: &str) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path))
}
```

### Step 2: Add module in main.rs

In `storage-service/src/main.rs`:

```rust
mod protected_paths;
```

### Step 3: Update unmount method

In `storage-service/src/filesystems.rs`, in the `unmount` method, add check before kill_processes:

```rust
// In unmount method, before the existing kill_processes logic:

if kill_processes {
    // Check if this is a protected system path
    if crate::protected_paths::is_protected_path(&mount_point) {
        tracing::warn!(
            "Rejecting kill_processes on protected path: {}",
            mount_point
        );

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

    // ... existing kill_processes logic continues
}
```

### Verification

```bash
# Build
cargo build --package storage-service

# Test (requires root)
sudo ./target/debug/storage-service &
./target/debug/storage-ui

# Try to unmount / with kill_processes - should show error
```

---

## Phase 3: FSTools Consolidation

### Step 1: Add FilesystemToolInfo to storage-common

In `storage-common/src/lib.rs`:

```rust
/// Information about a filesystem formatting tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemToolInfo {
    pub fs_type: String,
    pub fs_name: String,
    pub command: String,
    pub package_hint: String,
    pub available: bool,
}
```

### Step 2: Enhance FilesystemsHandler

In `storage-service/src/filesystems.rs`:

```rust
use storage_common::FilesystemToolInfo;

pub struct FilesystemsHandler {
    supported_tools: Vec<String>,           // Existing
    filesystem_tools: Vec<FilesystemToolInfo>, // New
}

impl FilesystemsHandler {
    pub fn new() -> Self {
        let filesystem_tools = Self::detect_all_filesystem_tools();
        let supported_tools = filesystem_tools
            .iter()
            .filter(|t| t.available)
            .map(|t| t.fs_type.clone())
            .collect();

        Self { supported_tools, filesystem_tools }
    }

    fn detect_all_filesystem_tools() -> Vec<FilesystemToolInfo> {
        let tools = vec![
            ("ext4", "EXT4", "mkfs.ext4", "e2fsprogs"),
            ("xfs", "XFS", "mkfs.xfs", "xfsprogs"),
            ("btrfs", "Btrfs", "mkfs.btrfs", "btrfs-progs"),
            ("vfat", "FAT32", "mkfs.vfat", "dosfstools"),
            ("ntfs", "NTFS", "mkfs.ntfs", "ntfs-3g"),
            ("exfat", "exFAT", "mkfs.exfat", "exfatprogs"),
            ("f2fs", "F2FS", "mkfs.f2fs", "f2fs-tools"),
            ("udf", "UDF", "mkudffs", "udftools"),
        ];

        tools.into_iter()
            .map(|(fs_type, fs_name, command, package)| FilesystemToolInfo {
                fs_type: fs_type.to_string(),
                fs_name: fs_name.to_string(),
                command: command.to_string(),
                package_hint: package.to_string(),
                available: which::which(command).is_ok(),
            })
            .collect()
    }
}
```

### Step 3: Add D-Bus method

```rust
#[interface(name = "org.cosmic.ext.StorageService.Filesystems")]
impl FilesystemsHandler {
    // ... existing methods ...

    /// Get detailed filesystem tool information
    async fn get_filesystem_tools(&self) -> zbus::fdo::Result<String> {
        serde_json::to_string(&self.filesystem_tools)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }
}
```

### Step 4: Add client method

In `storage-ui/src/client/filesystems.rs`:

```rust
pub async fn get_filesystem_tools(&self) -> Result<Vec<FilesystemToolInfo>, ClientError> {
    let json = self.proxy.get_filesystem_tools().await?;
    let tools: Vec<FilesystemToolInfo> = serde_json::from_str(&json)
        .map_err(|e| ClientError::ParseError(e.to_string()))?;
    Ok(tools)
}
```

### Step 5: Deprecate UI fs_tools.rs

Mark `storage-ui/src/utils/fs_tools.rs` as deprecated:

```rust
//! DEPRECATED: Use FilesystemsClient::get_filesystem_tools() instead
```

### Verification

```bash
# Build
cargo build --workspace

# Test service method
busctl --system call org.cosmic.ext.StorageService \
    /org/cosmic/ext/StorageService/filesystems \
    org.cosmic.ext.StorageService.Filesystems \
    get_filesystem_tools

# Run tests
cargo test --workspace --all-features
```

---

## Final Verification

```bash
# All quality gates
cargo fmt --all --check
cargo clippy --workspace --all-features
cargo test --workspace --all-features

# Integration test
sudo systemctl restart cosmic-ext-storage-service
./target/debug/storage-ui
```

## Common Issues

| Issue | Solution |
|-------|----------|
| Connection fails on first call | Normal - subsequent calls succeed |
| Protected path check fails for symlinks | Ensure `canonicalize` is working |
| FSTools not detected | Check `which::which` can find the command in PATH |

---

## APPENDIX: Phase 0 - Storage-DBus Connection Caching (Critical)

*Added during planning phase: Layer 2 (storage-dbus → UDisks2) connection caching.*

This phase addresses the most significant performance bottleneck: `get_disks_with_volumes()` creating a new D-Bus connection on every call (9+ times in service).

### Step 1: Add connection field to DiskManager

In `storage-dbus/src/disk/manager.rs`:

```rust
use std::sync::Arc;
use zbus::Connection;

pub struct DiskManager {
    /// Cached D-Bus connection to system bus (for UDisks2)
    connection: Arc<Connection>,

    // ... existing fields
}

impl DiskManager {
    pub async fn new() -> Result<Self, DiskError> {
        // Establish connection eagerly at creation
        let connection = Arc::new(
            Connection::system()
                .await
                .map_err(|e| DiskError::Connection(e.to_string()))?
        );

        Ok(Self {
            connection,
            // ... existing fields
        })
    }

    /// Get a reference to the cached connection for reuse
    pub fn connection(&self) -> &Arc<Connection> {
        &self.connection
    }

    // ... existing methods
}
```

### Step 2: Update discovery function signature

In `storage-dbus/src/disk/discovery.rs`, change the function to accept a manager reference:

```rust
// Before (creates new connection each time)
pub async fn get_disks_with_volumes() -> Result<Vec<(DiskInfo, Vec<VolumeInfo>)>> {
    let connection = Connection::system().await?;  // REMOVE THIS
    // ...
}

// After (reuses manager's cached connection)
pub async fn get_disks_with_volumes(
    manager: &DiskManager
) -> Result<Vec<(DiskInfo, Vec<VolumeInfo>)>, DiskError> {
    let connection = manager.connection();  // USE CACHED CONNECTION

    // Create proxies from shared connection
    let manager_proxy = UDisks2ManagerProxy::new(connection.as_ref()).await?;

    // ... rest of function unchanged
}
```

### Step 3: Update all call sites

In `storage-service/src/disks.rs`, update all calls to `get_disks_with_volumes()`:

```rust
// Before
let disks = get_disks_with_volumes().await?;

// After
let disks = get_disks_with_volumes(&self.manager).await?;
```

**Locations to update** (based on code exploration):
- `disks.rs` - multiple locations (search for `get_disks_with_volumes`)

### Step 4: Update function exports

Ensure the function is properly exported from `storage-dbus`:

```rust
// storage-dbus/src/disk/mod.rs
pub use discovery::get_disks_with_volumes;
```

### Verification

```bash
# Build
cargo build --workspace

# Add debug logging to verify connection reuse
# In get_disks_with_volumes, add:
tracing::debug!("Using cached connection for disk discovery");

# Run service and check logs
sudo ./target/debug/storage-service
# Should see "Using cached connection" multiple times without new connection establishment
```

---

## Revised Implementation Order

**Recommended order based on impact analysis:**

1. **Phase 0: Layer 2 Connection Caching** - BIGGEST IMPACT
   - storage-dbus → UDisks2 connection reuse
   - Affects ALL disk operations (9+ calls per operation)

2. **Phase 1: Layer 1 Connection Sharing**
   - storage-ui → storage-service connection reuse
   - Improves UI startup and responsiveness

3. **Phase 2: Protected Paths** - SAFETY CRITICAL
   - Prevents accidental system damage

4. **Phase 3: FSTools Consolidation**
   - Maintainability improvement

---

## Performance Metrics to Track

| Metric | Before | Target |
|--------|--------|--------|
| App startup time | ~5-10s | <3s |
| First disk enumeration | ~2-3s | <1s |
| Subsequent enumerations | ~2-3s (new conn each) | <200ms (cached) |
| UI event response | ~1-2s | <500ms |
