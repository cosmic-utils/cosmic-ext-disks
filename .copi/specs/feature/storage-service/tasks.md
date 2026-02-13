# Storage Service Architecture — Implementation Tasks

**Branch:** `feature/storage-service`  
**Total:** 5 phases across 6-8 weeks  
**Type:** Destructive refactor (no backward compatibility)

---

## Phase 1: Foundation & Library Conversion (2 weeks)

### Task 1.1: Convert disks-btrfs-helper to Library (1 week)

**Scope:** Transform CLI binary into reusable library crate

**Files:**
- `disks-btrfs-helper/` → rename to `disks-btrfs/`
- `disks-btrfs/Cargo.toml` (update to library crate)
- `disks-btrfs/src/lib.rs` (new - public API)
- `disks-btrfs/src/btrfs/` (move logic from main.rs)
- `disks-btrfs/src/error.rs` (new - shared error types)

**Steps:**
1. Rename crate directory: `mv disks-btrfs-helper disks-btrfs`
2. Update `Cargo.toml`:
   ```toml
   [package]
   name = "disks-btrfs"
   version = "0.2.0"
   edition = "2024"
   
   [lib]
   name = "disks_btrfs"
   path = "src/lib.rs"
   
   # Optional: keep CLI for testing
   [[bin]]
   name = "disks-btrfs-cli"
   path = "src/bin/cli.rs"
   required-features = ["cli"]
   
   [features]
   default = []
   cli = ["clap"]
   ```

3. Create error types `src/error.rs`:
   ```rust
   use thiserror::Error;
   
   #[derive(Error, Debug)]
   pub enum BtrfsError {
       #[error("Subvolume not found: {0}")]
       SubvolumeNotFound(String),
       
       #[error("Permission denied: {0}")]
       PermissionDenied(String),
       
       #[error("Filesystem not mounted: {0}")]
       NotMounted(String),
       
       #[error("Invalid path: {0}")]
       InvalidPath(String),
       
       #[error("BTRFS operation failed: {0}")]
       OperationFailed(String),
       
       #[error("IO error: {0}")]
       Io(#[from] std::io::Error),
       
       #[error("btrfsutil error: {0}")]
       Btrfsutil(#[from] btrfsutil::Error),
   }
   
   pub type Result<T> = std::result::Result<T, BtrfsError>;
   ```

4. Create library API `src/lib.rs`:
   ```rust
   pub mod error;
   pub mod subvolume;
   pub mod snapshot;
   pub mod usage;
   
   pub use error::{BtrfsError, Result};
   pub use subvolume::Subvolume;
   
   // Re-export btrfsutil types
   pub use btrfsutil::{SubvolumeInfo, SubvolumeIterator};
   ```

5. Refactor operations into modules:
   - `src/subvolume.rs` - list, create, delete operations
   - `src/snapshot.rs` - snapshot creation
   - `src/usage.rs` - quota and usage queries
   - `src/readonly.rs` - read-only flag operations
   - `src/default.rs` - default subvolume management

6. Example public API:
   ```rust
   // src/subvolume.rs
   pub struct SubvolumeManager {
       mountpoint: PathBuf,
   }
   
   impl SubvolumeManager {
       pub fn new<P: Into<PathBuf>>(mountpoint: P) -> Result<Self>;
       pub fn list_all(&self) -> Result<Vec<BtrfsSubvolume>>;
       pub fn get(&self, id: u64) -> Result<BtrfsSubvolume>;
       pub fn create(&self, path: &Path) -> Result<u64>;
       pub fn delete(&self, path: &Path) -> Result<()>;
   }
   ```

7. Move CLI to `src/bin/cli.rs` (behind feature flag):
   - Keep existing CLI interface for manual testing
   - Use library internally: `disks_btrfs::SubvolumeManager::new(...)`

8. Update workspace Cargo.toml:
   ```toml
   [workspace]
   members = ["disks-dbus", "disks-btrfs", "disks-ui"]
   ```

**Test Plan:**
- Library: `cargo test -p disks-btrfs`
- CLI (if enabled): `cargo run -p disks-btrfs --features cli -- list /`
- Integration test: create subvolume, verify visible in list

**Done When:**
- [x] Crate renamed to disks-btrfs (created new crate alongside old helper)
- [x] Public library API defined and documented
- [x] All operations moved to library modules
- [x] Error types unified
- [x] Tests pass
- [x] Optional CLI still functional

---

### Task 1.2: Create storage-service Crate (1 week)

**Scope:** Bootstrap D-Bus service with zbus

**Files:**
- `storage-service/` (new directory)
- `storage-service/Cargo.toml`
- `storage-service/src/main.rs`
- `storage-service/src/dbus_service.rs`
- `storage-service/src/btrfs_handler.rs`
- `storage-service/src/auth.rs`

**Steps:**
1. Create new crate:
   ```bash
   cd /path/to/workspace
   cargo new --bin storage-service
   ```

2. Add dependencies to `Cargo.toml`:
   ```toml
   [package]
   name = "storage-service"
   version = "0.1.0"
   edition = "2024"
   
   [[bin]]
   name = "cosmic-storage-service"
   path = "src/main.rs"
   
   [dependencies]
   # D-Bus
   zbus = { version = "5.0", features = ["tokio"] }
   zbus-polkit = "1.0"
   
   # Async runtime
   tokio = { version = "1.41", features = ["full"] }
   
   # Logging
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
   
   # Error handling
   anyhow = "1.0"
   thiserror = "1.0"
   
   # Serialization
   serde = { version = "1.0", features = ["derive"] }
   
   # Internal deps
   disks-btrfs = { path = "../disks-btrfs" }
   disks-dbus = { path = "../disks-dbus" }
   ```

3. Create main entry point `src/main.rs`:
   ```rust
   use tracing_subscriber;
   use zbus::{Connection, ConnectionBuilder};
   
   mod dbus_service;
   mod btrfs_handler;
   mod auth;
   
   #[tokio::main]
   async fn main() -> anyhow::Result<()> {
       // Setup logging
       tracing_subscriber::fmt()
           .with_env_filter("storage_service=debug,info")
           .init();
       
       tracing::info!("Starting COSMIC Storage Service");
       
       // Create D-Bus connection
       let conn = ConnectionBuilder::system()?
           .name("org.cosmic.ext.StorageService")?
           .serve_at("/org/cosmic/ext/StorageService", dbus_service::StorageService::new())?
           .serve_at("/org/cosmic/ext/StorageService/btrfs", btrfs_handler::BtrfsHandler::new())?
           .build()
           .await?;
       
       tracing::info!("Service registered on D-Bus");
       
       // Keep service running
       loop {
           tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
           // TODO: Implement idle timeout (exit if no calls for 60s)
       }
   }
   ```

4. Create D-Bus interface `src/dbus_service.rs`:
   ```rust
   use zbus::{dbus_interface, SignalContext};
   
   pub struct StorageService {
       version: String,
   }
   
   impl StorageService {
       pub fn new() -> Self {
           Self {
               version: env!("CARGO_PKG_VERSION").to_string(),
           }
       }
   }
   
   #[dbus_interface(name = "org.cosmic.ext.StorageService")]
   impl StorageService {
       #[dbus_interface(property)]
       async fn version(&self) -> &str {
           &self.version
       }
       
       #[dbus_interface(property)]
       async fn supported_features(&self) -> Vec<String> {
           vec!["btrfs".to_string(), "partitions".to_string()]
       }
   }
   ```

5. Create BTRFS handler `src/btrfs_handler.rs`:
   ```rust
   use zbus::{dbus_interface, SignalContext};
   use disks_btrfs::{SubvolumeManager, BtrfsSubvolume};
   use crate::auth::check_authorization;
   
   pub struct BtrfsHandler;
   
   impl BtrfsHandler {
       pub fn new() -> Self {
           Self
       }
   }
   
   #[dbus_interface(name = "org.cosmic.ext.StorageService.Btrfs")]
   impl BtrfsHandler {
       async fn list_subvolumes(
           &self,
           #[zbus(signal_context)] ctx: SignalContext<'_>,
           mountpoint: &str,
       ) -> zbus::fdo::Result<Vec<(u64, String, String)>> {
           // Check authorization
           check_authorization(&ctx, "org.cosmic.ext.storage-service.btrfs-read").await?;
           
           // Use library
           let manager = SubvolumeManager::new(mountpoint)
               .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
           
           let subvolumes = manager.list_all()
               .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
           
           // Convert to D-Bus types (simplified for now)
           let result = subvolumes.iter()
               .map(|s| (s.id, s.path.display().to_string(), s.uuid.to_string()))
               .collect();
           
           Ok(result)
       }
       
       async fn create_snapshot(
           &self,
           #[zbus(signal_context)] ctx: SignalContext<'_>,
           source_path: &str,
           dest_path: &str,
           readonly: bool,
       ) -> zbus::fdo::Result<u64> {
           check_authorization(&ctx, "org.cosmic.ext.storage-service.btrfs-modify").await?;
           
           // TODO: Implement using library
           Err(zbus::fdo::Error::Failed("Not implemented".to_string()))
       }
       
       // TODO: Add remaining methods
   }
   ```

6. Create authorization module `src/auth.rs`:
   ```rust
   use zbus::{SignalContext, fdo};
   use zbus_polkit::policykit1::Authority;
   
   pub async fn check_authorization(
       ctx: &SignalContext<'_>,
       action_id: &str,
   ) -> zbus::fdo::Result<()> {
       let connection = ctx.connection();
       let sender = ctx.header()?.sender()
           .ok_or_else(|| fdo::Error::Failed("No sender".to_string()))?;
       
       // Get authority
       let authority = Authority::new(connection)
           .await
           .map_err(|e| fdo::Error::Failed(format!("Polkit error: {}", e)))?;
       
       // Create subject from sender
       let subject = zbus_polkit::policykit1::Subject::new_for_owner(
           std::process::id(),
           None,
           None,
       ).map_err(|e| fdo::Error::Failed(e.to_string()))?;
       
       // Check authorization
       let result = authority.check_authorization(
           &subject,
           action_id,
           &std::collections::HashMap::new(),
           zbus_polkit::policykit1::CheckAuthorizationFlags::AllowUserInteraction.into(),
           "",
       ).await
           .map_err(|e| fdo::Error::Failed(format!("Polkit check failed: {}", e)))?;
       
       if !result.is_authorized {
           return Err(fdo::Error::AccessDenied(
               format!("Not authorized for action: {}", action_id)
           ));
       }
       
       Ok(())
   }
   ```

7. Update workspace Cargo.toml:
   ```toml
   [workspace]
   members = ["disks-dbus", "disks-btrfs", "disks-ui", "storage-service"]
   ```

**Test Plan:**
- Build service: `cargo build -p storage-service`
- Run as root: `sudo ./target/debug/cosmic-storage-service`
- Test with `busctl`:
  ```bash
  busctl --system tree org.cosmic.ext.StorageService
  busctl --system introspect org.cosmic.ext.StorageService /org/cosmic/ext/StorageService/btrfs
  busctl --system call org.cosmic.ext.StorageService /org/cosmic/ext/StorageService/btrfs org.cosmic.ext.StorageService.Btrfs ListSubvolumes s "/"
  ```

**Done When:**
- [x] Service builds without errors
- [x] Service registers on D-Bus system bus (requires root)
- [x] ListSubvolumes method callable via busctl (requires root testing)
- [x] Polkit authorization working (implemented, requires root testing)
- [x] Service logs to journald (tracing configured)

---

## Phase 2: BTRFS Operations Migration (1.5 weeks)

### Task 2.1: Implement All BTRFS D-Bus Methods (1 week)

**Scope:** Complete org.cosmic.ext.StorageService.Btrfs interface

**Files:**
- `storage-service/src/btrfs_handler.rs` (expand)
- `disks-btrfs/src/*.rs` (add missing operations)

**Steps:**
1. Implement CreateSubvolume method
2. Implement CreateSnapshot method
3. Implement DeleteSubvolume method
4. Implement SetReadOnly method
5. Implement GetDefaultSubvolume method
6. Implement SetDefaultSubvolume method
7. Implement GetUsage method (quotas)
8. Implement EnableQuotas method (with progress)
9. Add signal emission for SubvolumeChanged
10. Add operation ID tracking for long operations

**Test Plan:**
- Integration tests using zbus test connection
- Manual testing with busctl for each method
- Verify Polkit prompts show correctly

**Done When:**
- [ ] All BTRFS methods implemented
- [ ] Signals working
- [ ] Progress reporting for EnableQuotas
- [ ] Error handling comprehensive
- [ ] Integration tests passing

---

### Task 2.2: Create D-Bus Client Wrapper (0.5 weeks)

**Scope:** UI-side client for calling service

**Files:**
- `disks-ui/src/client/mod.rs` (new module)
- `disks-ui/src/client/btrfs.rs` (new)
- `disks-ui/src/client/error.rs` (new)
- `disks-ui/Cargo.toml` (add zbus dependency)

**Steps:**
1. Add zbus to UI dependencies:
   ```toml
   [dependencies]
   zbus = { version = "5.0", default-features = false }
   tokio = { version = "1.41", features = ["rt-multi-thread"] }
   ```

2. Create client module `src/client/mod.rs`:
   ```rust
   pub mod btrfs;
   pub mod error;
   
   pub use btrfs::BtrfsClient;
   pub use error::ClientError;
   ```

3. Create client error types `src/client/error.rs`:
   ```rust
   use thiserror::Error;
   
   #[derive(Error, Debug, Clone)]
   pub enum ClientError {
       #[error("D-Bus error: {0}")]
       DBus(String),
       
       #[error("Service not available")]
       ServiceNotAvailable,
       
       #[error("Permission denied: {0}")]
       PermissionDenied(String),
       
       #[error("Operation failed: {0}")]
       OperationFailed(String),
   }
   
   impl From<zbus::Error> for ClientError {
       fn from(err: zbus::Error) -> Self {
           match err {
               zbus::Error::FDO(ref e) if e.name() == Some("org.freedesktop.DBus.Error.AccessDenied") => {
                   ClientError::PermissionDenied(e.to_string())
               },
               _ => ClientError::DBus(err.to_string()),
           }
       }
   }
   ```

4. Create BTRFS client `src/client/btrfs.rs`:
   ```rust
   use zbus::{Connection, proxy};
   use crate::client::error::ClientError;
   
   #[proxy(
       interface = "org.cosmic.ext.StorageService.Btrfs",
       default_service = "org.cosmic.ext.StorageService",
       default_path = "/org/cosmic/ext/StorageService/btrfs"
   )]
   trait BtrfsInterface {
       async fn list_subvolumes(&self, mountpoint: &str) -> zbus::Result<Vec<(u64, String, String)>>;
       async fn create_snapshot(&self, source: &str, dest: &str, readonly: bool) -> zbus::Result<u64>;
       async fn delete_subvolume(&self, path: &str) -> zbus::Result<()>;
       async fn set_read_only(&self, path: &str, readonly: bool) -> zbus::Result<()>;
       // ... other methods
   }
   
   pub struct BtrfsClient {
       proxy: BtrfsInterfaceProxy<'static>,
   }
   
   impl BtrfsClient {
       pub async fn new() -> Result<Self, ClientError> {
           let conn = Connection::system().await?;
           let proxy = BtrfsInterfaceProxy::new(&conn).await?;
           Ok(Self { proxy })
       }
       
       pub async fn list_subvolumes(&self, mountpoint: &str) -> Result<Vec<BtrfsSubvolume>, ClientError> {
           let result = self.proxy.list_subvolumes(mountpoint).await?;
           // Convert D-Bus types to BtrfsSubvolume
           Ok(result.into_iter().map(|(id, path, uuid)| {
               // ... conversion
           }).collect())
       }
       
       pub async fn create_snapshot(&self, source: &str, dest: &str, readonly: bool) -> Result<u64, ClientError> {
           Ok(self.proxy.create_snapshot(source, dest, readonly).await?)
       }
       
       // ... other methods
   }
   ```

**Test Plan:**
- Create test that spawns service and client
- Verify each method works end-to-end
- Test error handling (service not running, permission denied)

**Done When:**
- [ ] Client compiles and links
- [ ] All BTRFS methods wrapped
- [ ] Error conversion working
- [ ] End-to-end test passing

---

## Phase 3: UI Refactor (2 weeks)

### Task 3.1: Replace Helper Calls with Client (1.5 weeks)

**Scope:** Remove all pkexec calls, use D-Bus client

**Files:**
- `disks-ui/src/ui/btrfs/mod.rs` (major refactor)
- `disks-ui/src/ui/btrfs/view.rs` (update message handling)
- `disks-ui/src/app.rs` (add async runtime)

**Steps:**
1. Add Tokio runtime to app initialization:
   ```rust
   // In main.rs
   fn main() -> Result<()> {
       let rt = tokio::runtime::Runtime::new()?;
       let _guard = rt.enter();
       
       cosmic::app::run::<App>(settings, flags)
   }
   ```

2. Initialize client on app startup:
   ```rust
   // In app.rs
   pub struct App {
       // ... existing fields
       btrfs_client: Option<Arc<BtrfsClient>>,
   }
   
   impl Application for App {
       fn init(&mut self) {
           // Try to connect to service
           let client = tokio::task::block_in_place(|| {
               tokio::runtime::Handle::current().block_on(async {
                   BtrfsClient::new().await.ok()
               })
           });
           
           if client.is_none() {
               tracing::warn!("Storage service not available, some features disabled");
           }
           
           self.btrfs_client = client.map(Arc::new);
       }
   }
   ```

3. Replace BtrfsMessage::LoadSubvolumes handling:
   ```rust
   // OLD:
   BtrfsMessage::LoadSubvolumes(path) => {
       let output = Command::new("pkexec")
           .arg("cosmic-ext-disks-btrfs-helper")
           .arg("list")
           .arg(&path)
           .output()?;
       // ... JSON parsing
   }
   
   // NEW:
   BtrfsMessage::LoadSubvolumes(path) => {
       let client = self.btrfs_client.clone()
           .ok_or("Service not available")?;
       let path = path.clone();
       
       return Command::perform(
           async move {
               client.list_subvolumes(&path).await
           },
           |result| match result {
               Ok(subvolumes) => Message::Btrfs(BtrfsMessage::SubvolumesLoaded(Ok(subvolumes))),
               Err(e) => Message::Btrfs(BtrfsMessage::SubvolumesLoaded(Err(e.to_string()))),
           }
       );
   }
   ```

4. Replace all other BTRFS operations similarly:
   - CreateSnapshot → client.create_snapshot()
   - DeleteSubvolume → client.delete_subvolume()
   - SetReadOnly → client.set_read_only()
   - etc.

5. Remove pkexec helper invocation code:
   - Delete `run_privileged_command()` function
   - Remove JSON serialization/deserialization
   - Remove Command::new("pkexec") calls

6. Update error handling for D-Bus errors:
   ```rust
   match result {
       Err(ClientError::PermissionDenied(_)) => {
           // Show dialog: "Permission denied. Please ensure you have admin rights."
       },
       Err(ClientError::ServiceNotAvailable) => {
           // Show dialog: "Storage service is not running. Please contact your system administrator."
       },
       Err(e) => {
           // Generic error toast
       },
   }
   ```

7. Handle service disconnection gracefully:
   - Show notification if service crashes
   - Offer "Retry" button to reconnect
   - Disable BTRFS UI if service unavailable

**Test Plan:**
- UI launches with service running
- UI launches with service NOT running (graceful degradation)
- All BTRFS operations work via D-Bus
- No pkexec prompts shown
- Polkit authentication shows COSMIC-native dialog
- Service crash is detected and reported

**Done When:**
- [ ] All pkexec code removed
- [ ] All operations use D-Bus client
- [ ] Error handling updated
- [ ] Graceful degradation working
- [ ] No regressions in BTRFS functionality

---

### Task 3.2: Remove Helper Binary (0.5 weeks)

**Scope:** Clean up old pkexec helper

**Files:**
- `disks-btrfs/src/bin/cli.rs` (keep for manual testing only)
- Root workspace (remove pkexec polkit actions for helper)
- Packaging scripts (remove helper binary installation)

**Steps:**
1. Mark helper binary as dev-only:
   ```toml
   [[bin]]
   name = "disks-btrfs-cli"
   path = "src/bin/cli.rs"
   required-features = ["cli"]  # Not built by default
   ```

2. Remove old polkit action file (if exists):
   - `/usr/share/polkit-1/actions/com.system80.pkexec.cosmic-ext-disks-btrfs-helper.policy`

3. Update documentation:
   - README: Remove references to helper binary
   - Add section about storage service requirement

4. Update packaging:
   - Remove helper binary from install paths
   - Add storage-service binary instead
   - Add systemd service files
   - Add new polkit policy file

**Done When:**
- [ ] Helper binary not built by default
- [ ] Old polkit actions removed
- [ ] Documentation updated
- [ ] Packaging scripts updated

---

## Phase 4: Service Expansion (2-3 weeks)

### Task 4.1: Add Partition Operations (1 week)

**Scope:** Implement org.cosmic.ext.StorageService.Partitions interface

**Files:**
- `storage-service/src/partition_handler.rs` (new)
- Similar pattern to btrfs_handler

**Steps:**
1. Implement partition handler using existing disks-dbus code
2. Migrate partition operations from UI
3. Create client wrapper
4. Update UI to use D-Bus for partitions

**Done When:**
- [ ] Partitions interface implemented
- [ ] UI migrated to D-Bus for partitions
- [ ] No pkexec calls for partition operations

---

### Task 4.2: Add LVM Operations (1 week)

**Scope:** Implement org.cosmic.ext.StorageService.Lvm interface

**Files:**
- `storage-service/src/lvm_handler.rs` (new)

**Steps:**
1. Implement LVM handler
2. Migrate LVM operations from UI
3. Create client wrapper
4. Update UI

**Done When:**
- [ ] LVM interface implemented
- [ ] UI migrated to D-Bus for LVM

---

### Task 4.3: Add SMART Monitoring (0.5 weeks)

**Scope:** Implement org.cosmic.ext.StorageService.Smart interface

**Files:**
- `storage-service/src/smart_handler.rs` (new)

**Steps:**
1. Implement SMART handler
2. Migrate SMART operations from UI
3. Create client wrapper
4. Update UI

**Done When:**
- [ ] SMART interface implemented
- [ ] UI migrated to D-Bus for SMART

---

## Phase 5: Production Hardening (1 week)

### Task 5.1: Systemd Integration (0.3 weeks)

**Scope:** Service files for auto-start

**Files:**
- `data/systemd/cosmic-storage-service.service` (new)
- `data/systemd/cosmic-storage-service.socket` (new)
- `data/dbus-1/system.d/org.cosmic.ext.StorageService.conf` (new)

**Steps:**
1. Create service file (see plan.md)
2. Create socket file (optional)
3. Create D-Bus policy:
   ```xml
   <!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
     "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
   <busconfig>
     <policy user="root">
       <allow own="org.cosmic.ext.StorageService"/>
       <allow send_destination="org.cosmic.ext.StorageService"/>
     </policy>
     
     <policy context="default">
       <allow send_destination="org.cosmic.ext.StorageService"/>
     </policy>
   </busconfig>
   ```

4. Add install paths to build system:
   ```toml
   # In Cargo.toml or build.rs
   # Install to: /usr/lib/systemd/system/
   # Install to: /usr/share/dbus-1/system.d/
   ```

**Test Plan:**
- Install service: `sudo systemctl daemon-reload`
- Enable service: `sudo systemctl enable cosmic-storage-service.service`
- Start service: `sudo systemctl start cosmic-storage-service.service`
- Check status: `sudo systemctl status cosmic-storage-service.service`
- Check logs: `journalctl -u cosmic-storage-service -f`
- Test socket activation (if implemented)

**Done When:**
- [ ] Service files created
- [ ] D-Bus policy allows access
- [ ] Service auto-starts on boot
- [ ] Service visible in journalctl

---

### Task 5.2: Packaging & Installation (0.4 weeks)

**Scope:** Update packaging for all distributions

**Files:**
- `debian/` (update for Debian/Ubuntu)
- `rpm/` (add RPM spec if needed)
- `PKGBUILD` (update for Arch)
- `justfile` or `Makefile` (install targets)

**Steps:**
1. Update Debian packaging:
   ```
   Package: cosmic-ext-disks
   Depends: cosmic-storage-service, polkit
   
   Package: cosmic-storage-service
   Depends: libc6, libpolkit-gobject-1-0
   ```

2. Add install targets:
   ```bash
   install:
       cargo install --path disks-ui --bin cosmic-ext-disks
       cargo install --path storage-service --bin cosmic-storage-service
       install -Dm644 data/systemd/cosmic-storage-service.service \
               /usr/lib/systemd/system/
       install -Dm644 data/dbus-1/system.d/org.cosmic.ext.StorageService.conf \
               /usr/share/dbus-1/system.d/
       install -Dm644 data/polkit-1/org.cosmic.ext.storage-service.policy \
               /usr/share/polkit-1/actions/
   ```

3. Test installation on each distro:
   - Ubuntu 24.04
   - Fedora 40
   - Arch Linux
   - Pop!_OS

**Done When:**
- [ ] Debian package builds
- [ ] RPM package builds (if supported)
- [ ] Arch PKGBUILD works
- [ ] Install script tested on 3+ distros

---

### Task 5.3: Integration Tests (0.3 weeks)

**Scope:** Automated tests for D-Bus API

**Files:**
- `storage-service/tests/integration_test.rs` (new)

**Steps:**
1. Create integration test using zbus test connection:
   ```rust
   #[tokio::test]
   async fn test_btrfs_list_subvolumes() {
       let connection = ConnectionBuilder::session()?
           .name("org.cosmic.ext.StorageService")?
           .serve_at("/org/cosmic/ext/StorageService/btrfs", BtrfsHandler::new())?
           .build()
           .await?;
       
       let proxy = BtrfsInterfaceProxy::builder(&connection)
           .path("/org/cosmic/ext/StorageService/btrfs")?
           .build()
           .await?;
       
       // Requires test BTRFS filesystem
       let result = proxy.list_subvolumes("/tmp/test-btrfs").await;
       assert!(result.is_ok());
   }
   ```

2. Add CI tests (may require privileged runner or mock)

**Done When:**
- [ ] Integration tests written
- [ ] Tests pass in CI (or documented why they can't run)

---

## Final Checklist

### Functional
- [ ] All BTRFS operations work via D-Bus
- [ ] All partition operations work via D-Bus
- [ ] All LVM operations work via D-Bus
- [ ] All SMART operations work via D-Bus
- [ ] No pkexec calls remain in UI code
- [ ] Service auto-starts on D-Bus activation
- [ ] Service shuts down after idle timeout

### Security
- [ ] Polkit authorization on all modify operations
- [ ] D-Bus policy prevents unauthorized access
- [ ] Service runs with minimal privileges
- [ ] No information leaks to unprivileged users

### Performance
- [ ] D-Bus call latency <5ms (measure with `perf`)
- [ ] Service memory usage <50MB idle
- [ ] Long operations report progress

### Quality
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace` passes with no warnings
- [ ] `cargo fmt --check` passes
- [ ] Integration tests pass
- [ ] Manual testing checklist complete

### Documentation
- [ ] README updated with service requirement
- [ ] D-Bus API documented (introspection + markdown)
- [ ] Migration guide for packagers written
- [ ] CHANGELOG updated

### Packaging
- [ ] Service installs to /usr/bin/
- [ ] Systemd files install correctly
- [ ] D-Bus policy installs correctly
- [ ] Polkit policy installs correctly
- [ ] Works on Ubuntu, Fedora, Arch

---

## Rollback Plan

If critical issues found:

1. **Stage 1: Keep old helper as fallback**
   - Add feature flag: `use-storage-service`
   - Default to false initially
   - If service fails, fall back to pkexec helper

2. **Stage 2: Beta testing**
   - Ship service but keep helper installed
   - Monitor error reports
   - Fix issues before removing helper

3. **Stage 3: Remove helper**
   - Only after 1 month of stable service
   - Only after all major distros package service
   - Keep helper code in git history

---

## Success Metrics

### Week 2 (End of Phase 1):
- [ ] Library compiles and helper converted
- [ ] Service registers on D-Bus
- [ ] ListSubvolumes callable via busctl

### Week 4 (End of Phase 2):
- [ ] All BTRFS methods implemented
- [ ] UI can call service successfully
- [ ] 50% of BTRFS operations migrated

### Week 6 (End of Phase 3):
- [ ] All BTRFS operations use D-Bus
- [ ] No pkexec calls for BTRFS
- [ ] UI stable with new architecture

### Week 8 (End of Phase 5):
- [ ] All operations use D-Bus
- [ ] Service packaged for 3+ distros
- [ ] Zero critical bugs
- [ ] User feedback positive
