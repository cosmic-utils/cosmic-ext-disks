# Storage Service Architecture — Implementation Spec

**Branch:** `feature/storage-service`  
**Type:** Major Refactor (Architectural Change)  
**Estimated Effort:** 6-8 weeks  
**Status:** Phase 1 & 3B Complete (Foundation + Abstraction Layers)  
**Breaking Change:** Yes (destructive UI refactor)

---

## Goal

Replace the current pkexec-based privilege escalation model with a proper D-Bus system service architecture for ALL disk operations (not just BTRFS).

**Core Transformation:**
1. **Rename `storage-btrfs-helper` → `storage-btrfs`** (convert CLI to library crate)
2. **Create new `storage-service` crate** (D-Bus service exposing all disk operations)
3. **Destructive UI refactor** (remove all legacy code, use D-Bus client exclusively)

---

## Problem Statement

### Current Architecture Problems

**1. Security Model Issues:**
- Every privileged operation requires pkexec prompt
- Users see: "Authentication required for cosmic-ext-storage-btrfs-helper"
- Poor UX: multiple prompts for batch operations
- No capability-based permissions
- Difficult to audit what operations are allowed

**2. Performance Issues:**
- Process spawn overhead for every operation
- JSON serialization/deserialization for every call
- No connection reuse or batching
- Cold-start penalty for each pkexec invocation

**3. Architecture Limitations:**
- Helper binary is tightly coupled to BTRFS
- No shared code between disk operations (LVM, partitions, SMART, etc.)
- Difficult to add new operations without more helper binaries
- No async/await - everything blocks UI thread
- Cannot monitor long-running operations (format, resize)

**4. Maintenance Burden:**
- Duplicate logic between UI and helper
- Error handling via JSON parsing is fragile
- No type safety across process boundary
- Testing requires root privileges
- No integration with systemd (can't auto-start, can't use service features)

---

## Proposed Architecture

### New Component Stack

```
┌─────────────────────────────────────────────────────────────┐
│                     cosmic-ext-disks (UI)                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  storage-ui/src/                                      │   │
│  │    ├── app.rs                                       │   │
│  │    ├── ui/ (view layer - no privileged ops)        │   │
│  │    └── client/ (D-Bus client wrapper) ←─────────┐  │   │
│  └─────────────────────────────────────────────────┘    │   │
└──────────────────────────────────────────│──────────────┘   │
                                           ▼                   │
                                    D-Bus System Bus           │
                                           │                   │
           ┌───────────────────────────────┴───────────────────┤
           │                                                   │
           ▼                                                   ▼
┌────────────────────────────────┐  ┌──────────────────────────┐
│  storage-service (systemd)     │  │  Other D-Bus Services    │
│  ┌──────────────────────────┐  │  │  - udisks2               │
│  │ D-Bus Interface Layer    │  │  │  - blockdev              │
│  │  org.cosmic.ext.StorageService│ │  └──────────────────────────┘
│  ├──────────────────────────┤  │
│  │ Operation Handlers       │  │
│  │  ├── btrfs_handler.rs    │  │
│  │  ├── partition_handler.rs│ │
│  │  ├── lvm_handler.rs      │  │
│  │  └── smart_handler.rs    │  │
│  ├──────────────────────────┤  │
│  │ Backend Libraries        │  │
│  │  ├── storage-btrfs (lib)   │  │ ← Converted from helper
│  │  ├── storage-dbus (lib)    │  │ ← Existing
│  │  ├── libblkid            │  │
│  │  └── libudev             │  │
│  └──────────────────────────┘  │
└────────────────────────────────┘
    │
    └─► Kernel APIs (ioctl, sysfs, etc.)
```

### Key Design Decisions

**1. D-Bus Instead of Polkit Actions:**
- **Current:** `pkexec cosmic-ext-storage-btrfs-helper snapshot ...`
- **Proposed:** UI → D-Bus method call → service checks authorization → execute
- **Benefits:**
  - Single authentication at app launch (not per-operation)
  - Fine-grained permissions via Polkit D-Bus integration
  - Better UX: no repeated password prompts

**2. Library-First Design:**
- Convert `storage-btrfs-helper` binary → `storage-btrfs` library crate
- Expose Rust API for all BTRFS operations
- Storage service uses library, not subprocess calls
- **Benefits:**
  - Type safety across service/UI boundary
  - Shared error types
  - Integration testing without root
  - Easier to unit test

**3. Async/Await Throughout:**
- Service uses Tokio runtime
- All D-Bus methods are async
- Long operations report progress via D-Bus signals
- UI can cancel operations
- **Benefits:**
  - Non-blocking UI
  - Progress bars for long operations
  - Cancellation support

**4. Systemd Integration:**
- Service: `cosmic-storage-service.service`
- Socket activation: `cosmic-storage-service.socket`
- Auto-start on first D-Bus call
- Idle timeout (exit after 60s inactivity)
- **Benefits:**
  - No manual service management
  - Minimal resource usage when idle
  - System integration (journald logs, restart policies)

---

## Scope

### Phase 1: Foundation (In Scope)
- ✅ Convert `storage-btrfs-helper` → `storage-btrfs` library crate
- ✅ Create `storage-service` crate with D-Bus skeleton
- ✅ Implement BTRFS operations in service (migrate from helper)
- ✅ Create D-Bus client wrapper in UI crate
- ✅ Polkit policy file for authorization

### Phase 2: Migration (In Scope)
- ✅ Refactor UI to use D-Bus client (BTRFS operations only)
- ✅ Remove pkexec helper code from UI
- ✅ Update error handling for D-Bus errors
- ✅ Add progress reporting for long operations

### Phase 3: Expansion (In Scope)
- ✅ Add partition operations to service (GPT, MBR)
- ✅ Add LVM operations to service
- ✅ Add SMART monitoring to service
- ✅ Add filesystem operations (format, resize, mount)

### Phase 4: Polish (In Scope)
- ✅ Systemd service files and socket activation
- ✅ D-Bus policy configuration
- ✅ Integration tests using D-Bus test bus
- ✅ Documentation (D-Bus API reference, architecture docs)

### Out of Scope (V3.0+)
- ❌ Multi-user support (service is system-global)
- ❌ Remote storage management (network block devices)
- ❌ Plugin architecture (service is monolithic for now)
- ❌ WebSocket/REST API (D-Bus only)

---

## D-Bus API Design

### Service Name
`org.cosmic.ext.StorageService`

### Object Paths
- `/org/cosmic/ext/StorageService` - Main service object
- `/org/cosmic/ext/StorageService/btrfs` - BTRFS operations
- `/org/cosmic/ext/StorageService/partitions` - Partition operations
- `/org/cosmic/ext/StorageService/lvm` - LVM operations
- `/org/cosmic/ext/StorageService/smart` - SMART monitoring

### Interfaces

#### org.cosmic.ext.StorageService.Btrfs

**Methods:**
```xml
<method name="ListSubvolumes">
  <arg name="mountpoint" type="s" direction="in"/>
  <arg name="subvolumes" type="a(tsssuuta{sv})" direction="out"/>
</method>

<method name="CreateSubvolume">
  <arg name="parent_path" type="s" direction="in"/>
  <arg name="name" type="s" direction="in"/>
  <arg name="subvolume_id" type="t" direction="out"/>
</method>

<method name="CreateSnapshot">
  <arg name="source_path" type="s" direction="in"/>
  <arg name="dest_path" type="s" direction="in"/>
  <arg name="readonly" type="b" direction="in"/>
  <arg name="subvolume_id" type="t" direction="out"/>
</method>

<method name="DeleteSubvolume">
  <arg name="path" type="s" direction="in"/>
</method>

<method name="SetReadOnly">
  <arg name="path" type="s" direction="in"/>
  <arg name="readonly" type="b" direction="in"/>
</method>

<method name="GetDefaultSubvolume">
  <arg name="mountpoint" type="s" direction="in"/>
  <arg name="subvolume_id" type="t" direction="out"/>
</method>

<method name="SetDefaultSubvolume">
  <arg name="mountpoint" type="s" direction="in"/>
  <arg name="subvolume_id" type="t" direction="in"/>
</method>

<method name="GetUsage">
  <arg name="mountpoint" type="s" direction="in"/>
  <arg name="usage" type="a(ttt)" direction="out"/>  <!-- id, referenced, exclusive -->
</method>

<method name="EnableQuotas">
  <arg name="mountpoint" type="s" direction="in"/>
  <arg name="operation_id" type="s" direction="out"/>  <!-- for progress tracking -->
</method>
```

**Signals:**
```xml
<signal name="OperationProgress">
  <arg name="operation_id" type="s"/>
  <arg name="current" type="t"/>
  <arg name="total" type="t"/>
  <arg name="status" type="s"/>
</signal>

<signal name="SubvolumeChanged">
  <arg name="path" type="s"/>
  <arg name="change_type" type="s"/>  <!-- "created", "deleted", "modified" -->
</signal>
```

**Properties:**
```xml
<property name="Version" type="s" access="read"/>
<property name="SupportedFeatures" type="as" access="read"/>
```

---

#### org.cosmic.ext.StorageService.Partitions

**Methods:**
```xml
<method name="ListPartitions">
  <arg name="device_path" type="s" direction="in"/>
  <arg name="partitions" type="aa{sv}" direction="out"/>
</method>

<method name="CreatePartition">
  <arg name="device_path" type="s" direction="in"/>
  <arg name="start_sector" type="t" direction="in"/>
  <arg name="end_sector" type="t" direction="in"/>
  <arg name="partition_type" type="s" direction="in"/>
  <arg name="name" type="s" direction="in"/>
  <arg name="partition_path" type="s" direction="out"/>
</method>

<method name="DeletePartition">
  <arg name="partition_path" type="s" direction="in"/>
</method>

<method name="ResizePartition">
  <arg name="partition_path" type="s" direction="in"/>
  <arg name="new_size_bytes" type="t" direction="in"/>
  <arg name="operation_id" type="s" direction="out"/>
</method>

<method name="SetPartitionType">
  <arg name="partition_path" type="s" direction="in"/>
  <arg name="type_guid" type="s" direction="in"/>
</method>
```

---

#### org.cosmic.ext.StorageService.Lvm

**Methods:**
```xml
<method name="ListVolumeGroups">
  <arg name="volume_groups" type="aa{sv}" direction="out"/>
</method>

<method name="CreateLogicalVolume">
  <arg name="vg_name" type="s" direction="in"/>
  <arg name="lv_name" type="s" direction="in"/>
  <arg name="size_bytes" type="t" direction="in"/>
  <arg name="lv_path" type="s" direction="out"/>
</method>

<!-- etc. -->
```

---

## Technology Stack

### Storage Service Dependencies
```toml
[dependencies]
# D-Bus framework
zbus = "5.0"           # Async D-Bus library
zbus-polkit = "1.0"    # Polkit integration for authorization

# Async runtime
tokio = { version = "1.41", features = ["full"] }

# Backend libraries
storage-btrfs = { path = "../storage-btrfs" }  # Converted library
storage-dbus = { path = "../storage-dbus" }     # Existing

# System integration
libudev = "0.3"        # Device monitoring
nix = "0.29"           # Safe ioctl wrappers

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"
```

### UI Client Dependencies (additions)
```toml
[dependencies]
# D-Bus client
zbus = "5.0"
tokio = { version = "1.41", features = ["rt-multi-thread"] }

# (Remove pkexec-related code)
```

---

## Polkit Integration

### Authorization Flow

1. **UI calls D-Bus method** (e.g., `CreateSnapshot`)
2. **Service receives call**, extracts caller UID from D-Bus message
3. **Service calls Polkit** via `zbus-polkit`:
   - Action: `org.cosmic.ext.storage-service.btrfs-modify`
   - Subject: Caller UID from D-Bus
   - Polkit resolves based on policy
4. **If authorized:** Execute operation, return result
5. **If not authorized:** Return D-Bus error `org.freedesktop.DBus.Error.AccessDenied`
6. **UI handles error:** Show "Permission denied" dialog with explanation

### Polkit Actions

File: `/usr/share/polkit-1/actions/org.cosmic.ext.storage-service.policy`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE policyconfig PUBLIC "-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN"
  "http://www.freedesktop.org/standards/PolicyKit/1/policyconfig.dtd">
<policyconfig>
  <vendor>System76</vendor>
  <vendor_url>https://github.com/cosmic-utils/disks</vendor_url>

  <!-- BTRFS Operations -->
  <action id="org.cosmic.ext.storage-service.btrfs-read">
    <description>Read BTRFS filesystem information</description>
    <message>Authentication is required to read BTRFS filesystem information</message>
    <defaults>
      <allow_any>no</allow_any>
      <allow_inactive>no</allow_inactive>
      <allow_active>yes</allow_active>  <!-- No auth for read operations -->
    </defaults>
  </action>

  <action id="org.cosmic.ext.storage-service.btrfs-modify">
    <description>Modify BTRFS filesystems</description>
    <message>Authentication is required to modify BTRFS filesystems</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin_keep</allow_active>  <!-- Auth once, remember for session -->
    </defaults>
  </action>

  <!-- Partition Operations -->
  <action id="org.cosmic.ext.storage-service.partition-read">
    <description>Read partition information</description>
    <message>Authentication is required to read partition information</message>
    <defaults>
      <allow_any>no</allow_any>
      <allow_inactive>no</allow_inactive>
      <allow_active>yes</allow_active>
    </defaults>
  </action>

  <action id="org.cosmic.ext.storage-service.partition-modify">
    <description>Modify disk partitions</description>
    <message>Authentication is required to modify disk partitions</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin_keep</allow_active>
    </defaults>
  </action>

  <!-- LVM Operations -->
  <action id="org.cosmic.ext.storage-service.lvm-modify">
    <description>Modify LVM volumes</description>
    <message>Authentication is required to modify LVM volumes</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin_keep</allow_active>
    </defaults>
  </action>

  <!-- Dangerous Operations (Always require auth) -->
  <action id="org.cosmic.ext.storage-service.format">
    <description>Format storage devices</description>
    <message>Authentication is required to format storage devices</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin</allow_active>  <!-- Always prompt for format -->
    </defaults>
  </action>
</policyconfig>
```

### Authorization in Code

```rust
use zbus::Connection;
use zbus_polkit::policykit1::Authority;

async fn check_authorization(
    conn: &Connection,
    sender: &str,
    action_id: &str,
) -> Result<bool, Error> {
    let authority = Authority::new(conn).await?;
    
    let subject = zbus_polkit::policykit1::Subject::new_for_owner(
        std::process::id(),
        None,
        None,
    )?;
    
    let result = authority.check_authorization(
        &subject,
        action_id,
        &std::collections::HashMap::new(),
        zbus_polkit::policykit1::CheckAuthorizationFlags::AllowUserInteraction.into(),
        "",
    ).await?;
    
    Ok(result.is_authorized)
}
```

---

## Systemd Integration

### Service File

File: `/usr/lib/systemd/system/cosmic-storage-service.service`

```ini
[Unit]
Description=COSMIC Storage Management Service
Documentation=https://github.com/cosmic-utils/disks
Requires=dbus.service
After=dbus.service

[Service]
Type=dbus
BusName=org.cosmic.ext.StorageService
ExecStart=/usr/bin/cosmic-storage-service
Restart=on-failure
RestartSec=5s

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/dev /sys /run/udev

# Resource limits
MemoryMax=256M
TasksMax=50

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=cosmic-storage-service

[Install]
WantedBy=multi-user.target
```

### Socket Activation (Optional)

File: `/usr/lib/systemd/system/cosmic-storage-service.socket`

```ini
[Unit]
Description=COSMIC Storage Management Service Socket
Documentation=https://github.com/cosmic-utils/disks

[Socket]
ListenStream=/run/cosmic-storage-service.sock
SocketMode=0660
SocketUser=root
SocketGroup=storage

[Install]
WantedBy=sockets.target
```

---

## Migration Strategy

### Phase 1: Parallel Development (2 weeks)
**Goal:** Build service, no UI changes yet

1. Create `storage-service` crate
2. Implement D-Bus skeleton with zbus
3. Convert `storage-btrfs-helper` → `storage-btrfs` library
4. Implement BTRFS operations in service using library
5. Add Polkit integration
6. Write integration tests using D-Bus test bus

**Outcome:** Service is functional, old helper still in use by UI

---

### Phase 2: UI Client Wrapper (1 week)
**Goal:** Create D-Bus client API in UI crate

1. Add `storage-ui/src/client/` module
2. Implement `BtrfsClient` using zbus:
   ```rust
   pub struct BtrfsClient {
       proxy: BtrfsProxy<'static>,
   }
   
   impl BtrfsClient {
       pub async fn list_subvolumes(&self, mountpoint: &str) -> Result<Vec<BtrfsSubvolume>>;
       pub async fn create_snapshot(&self, source: &str, dest: &str, readonly: bool) -> Result<u64>;
       // ... etc
   }
   ```
3. Add progress signal handlers
4. Add error mapping from D-Bus errors to UI errors

**Outcome:** Client API ready, not yet used by UI

---

### Phase 3: Destructive UI Refactor (2 weeks)
**Goal:** Remove all pkexec code, use D-Bus exclusively

1. Replace helper calls with client calls:
   - `BtrfsMessage::LoadSubvolumes` → `client.list_subvolumes().await`
   - `BtrfsMessage::CreateSnapshot` → `client.create_snapshot(...).await`
   - etc.
2. Remove `storage-btrfs-helper` binary crate (or mark deprecated)
3. Remove JSON serialization code
4. Update error handling
5. Add async message handling in UI (use Tokio runtime)
6. Handle D-Bus connection errors gracefully

**Outcome:** BTRFS operations use D-Bus, old helper removed

---

### Phase 4: Service Expansion (2-3 weeks)
**Goal:** Add partition, LVM, SMART operations

1. Implement partition handler in service
2. Implement LVM handler in service
3. Implement SMART handler in service
4. Create client wrappers for each
5. Migrate UI code to use new clients
6. Remove any remaining pkexec calls

**Outcome:** All privileged operations use D-Bus service

---

### Phase 5: Systemd & Packaging (1 week)
**Goal:** Production-ready deployment

1. Write systemd service and socket files
2. Write D-Bus policy configuration
3. Update packaging (Debian, RPM, Arch)
4. Add install/uninstall scripts
5. Document deployment for distributors

**Outcome:** Service can be installed and auto-starts

---

## Acceptance Criteria

### Functional Requirements
- [ ] **AC-1:** All BTRFS operations work via D-Bus (no pkexec calls)
- [ ] **AC-2:** Service auto-starts on first D-Bus call (systemd socket activation)
- [ ] **AC-3:** Single authentication per session (not per operation)
- [ ] **AC-4:** Long operations report progress via D-Bus signals
- [ ] **AC-5:** UI can cancel long-running operations
- [ ] **AC-6:** Service handles multiple concurrent clients
- [ ] **AC-7:** Polkit policies correctly enforce authorization
- [ ] **AC-8:** Service logs to journald with structured logging

### Security Requirements
- [ ] **AC-9:** No elevated privileges in UI process
- [ ] **AC-10:** Service runs as root but with minimal capabilities
- [ ] **AC-11:** All operations authorized via Polkit
- [ ] **AC-12:** D-Bus policy prevents unauthorized access
- [ ] **AC-13:** No information leaks to unprivileged users

### Performance Requirements
- [ ] **AC-14:** D-Bus method call overhead <5ms (vs ~50ms for pkexec)
- [ ] **AC-15:** Service idle shutdown after 60s of inactivity
- [ ] **AC-16:** Memory usage <50MB when idle, <256MB under load
- [ ] **AC-17:** Concurrent operations don't block each other

### Quality Requirements
- [ ] **AC-18:** `cargo test --workspace` passes
- [ ] **AC-19:** `cargo clippy --workspace` passes
- [ ] **AC-20:** D-Bus API has integration tests
- [ ] **AC-21:** Documentation includes D-Bus API reference
- [ ] **AC-22:** Migration guide for packagers

---

## Technical Constraints

### Must Have
- ✅ Linux-only (D-Bus is Linux ecosystem)
- ✅ Root privileges for service (disk operations require root)
- ✅ Polkit installed and running (for authorization)
- ✅ systemd (for service management)
- ✅ D-Bus system bus accessible

### Should Have
- ✅ Tokio async runtime (performance)
- ✅ zbus 5.0+ (modern D-Bus API)
- ✅ Structured logging (tracing crate)
- ✅ Error propagation via anyhow/thiserror

### Nice to Have
- 🔄 Socket activation (saves resources)
- 🔄 Capability-based security (reduce root privileges)
- 🔄 D-Bus introspection (auto-generated docs)

---

## Risk Mitigation

### Risk 1: Breaking Change (Destructive Refactor)
**Impact:** Existing UI code stops working during migration  
**Mitigation:**
- Phase 2 completes parallel client wrapper before touching UI
- Feature flag: `use-dbus-client` (default: false during development)
- Keep old helper binary until service is stable
- Can rollback by re-enabling helper code

### Risk 2: Polkit Configuration Issues
**Impact:** Users can't authenticate, operations fail  
**Mitigation:**
- Extensive testing on different distros (Ubuntu, Fedora, Arch)
- Clear error messages: "Polkit authentication failed - ensure polkit is running"
- Fallback documentation for manual polkit troubleshooting
- Test with different DE polkit agents (GNOME, KDE, COSMIC)

### Risk 3: D-Bus Performance
**Impact:** Service calls slower than expected  
**Mitigation:**
- Benchmark early (compare pkexec vs D-Bus latency)
- Batch operations where possible
- Use D-Bus signals for progress (not polling)
- Profile with `perf` and optimize hot paths

### Risk 4: Service Reliability
**Impact:** Service crashes, leaves system in inconsistent state  
**Mitigation:**
- Comprehensive error handling (no panics)
- Transactional operations where possible
- Service auto-restart via systemd
- Idempotent operations (safe to retry)
- Extensive integration testing

### Risk 5: Packaging Complexity
**Impact:** Difficult to package for different distros  
**Mitigation:**
- Provide example packaging files (Debian, RPM, Arch)
- Document required files (service, socket, polkit policy)
- Test installation on CI (container-based)
- Engage with distro maintainers early

---

## Success Metrics

### User Experience
- ✅ Users authenticate once per session (not per operation)
- ✅ Batch operations don't show multiple password prompts
- ✅ Long operations show progress bars (not frozen UI)
- ✅ Operations can be cancelled mid-flight

### Performance
- ✅ Operation latency reduced by 10x (50ms → 5ms)
- ✅ Service idle memory usage <50MB
- ✅ No noticeable UI lag during disk operations

### Reliability
- ✅ Zero service crashes in first month
- ✅ All operations succeed or fail gracefully
- ✅ No user data loss due to service issues

### Adoption
- ✅ Successfully packaged for 3+ major distros
- ✅ Positive feedback from beta testers
- ✅ No critical issues reported in first 3 months

---

## Dependencies

### New Crates
- `zbus = "5.0"` (D-Bus framework)
- `zbus-polkit = "1.0"` (Polkit integration)
- `tokio = "1.41"` (async runtime - may already exist in workspace)
- `tracing` / `tracing-subscriber` (structured logging)

### System Dependencies
- `dbus` (system bus daemon)
- `polkit` (authorization framework)
- `systemd` (service management)

### Internal Dependencies
- `storage-btrfs` (new library, converted from helper)
- `storage-dbus` (existing library)

---

## References

- **D-Bus Specification:** https://dbus.freedesktop.org/doc/dbus-specification.html
- **zbus Docs:** https://docs.rs/zbus
- **Polkit Manual:** https://www.freedesktop.org/software/polkit/docs/latest/
- **systemd Service:** https://www.freedesktop.org/software/systemd/man/systemd.service.html
- **Similar Projects:**
  - udisks2 (disk management D-Bus service)
  - NetworkManager (D-Bus service example)
  - flatpak (Polkit integration example)

---

## Future Enhancements (V3.0+)

### Multi-User Support
- Per-user quotas and permissions
- User-specific BTRFS subvolumes
- Session-based service instances

### Plugin Architecture
- Dynamic operation handlers
- Third-party extensions
- Custom filesystem support

### Remote Management
- D-Bus over network (using systemd socket proxy)
- REST API gateway (wrap D-Bus methods)
- Web UI for headless servers

### Advanced Features
- Operation queuing and scheduling
- Transactional multi-step operations
- Undo/redo support for destructive operations
- Automated snapshots via service (not just GUI)

---

## Implementation Notes

This is a **destructive refactor** - we are replacing the entire privilege escalation model. The old pkexec-based approach will be removed entirely.

Migration is **all-or-nothing** per component:
- Phase 3 removes pkexec for BTRFS (can't have both)
- Phase 4 removes pkexec for other operations
- No backward compatibility with old helper binary

See `tasks.md` for detailed implementation steps organized by phase.

### Progress Update (2026-02-14)

**Phase 1: Complete ✅**
- ✅ storage-btrfs library created (v0.2.0) with full BTRFS operations
- ✅ storage-service D-Bus daemon created with socket activation
- ✅ Polkit authorization integration (read/modify separation)
- ✅ systemd integration (service, socket, D-Bus config)

**Phase 3B: Complete ✅ (GAP-001)**
- ✅ Created storage-sys crate (low-level file I/O abstraction)
- ✅ Created storage-dbus operations module (15 UDisks2 abstraction operations)
- ✅ Refactored partitions.rs (100% delegated to operations)
- ✅ Refactored filesystems.rs (100% delegated to operations)
- ✅ Refactored luks.rs encryption operations (100% delegated)
- ✅ Refactored image.rs backup/restore (100% delegated to storage-sys)
- ✅ Verified btrfs.rs clean (uses storage-btrfs library)
- ✅ Architecture pattern achieved: auth → delegate → signal

**Architecture Achieved:**
```
storage-service (Layer 1: auth + orchestration, 5-14 lines per method)
    ├─→ storage-dbus operations (Layer 2: UDisks2 abstraction, 15 ops)
    │       └─→ udisks2 daemon (Layer 3: system integration)
    ├─→ storage-sys (Layer 2: file I/O abstraction, 2 ops)
    │       └─→ kernel (Layer 3: direct I/O)
    └─→ storage-btrfs (Layer 2: BTRFS abstraction, 9 ops)
            └─→ btrfsutil + CLI (Layer 3: BTRFS tools)
```

**Code Quality Metrics:**
- Lines removed from storage-service: ~500+ (proxy code)
- Lines added in abstraction layers: ~800
- Methods refactored: 22 (across 4 files)
- Build status: ✅ 0 errors (cargo build --workspace)

**Next Phases:**
- Phase 2: LVM operations (create/delete/resize VGs/LVs)
- Phase 3: UI integration with D-Bus client
- Phase 4-5: Testing, deployment, migration

See `implementation-log.md` for detailed session-by-session progress.
