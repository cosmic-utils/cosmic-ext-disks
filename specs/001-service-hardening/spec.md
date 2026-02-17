# Feature Specification: Service Hardening

**Feature Branch**: `001-service-hardening`
**Created**: 2026-02-15
**Status**: Draft
**Input**: User description: "Performance issues. Likely need to try and persist clients/proxys. Takes a long time on app startup, and to respond to events. We should prevent kill process requests via unmount on partitions/filesystems that are mounted to system paths like /, /boot, etc. This should return an error explaining it and surface it to the user via the dialog (use the helper). Move FSTools checks into service. Can use this for feature enablement on service -> main.rs and service.rs"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Fast Application Startup and Responsive UI (Priority: P1)

As a user of the storage management application, I want the application to start quickly and respond promptly to my actions, so that I can manage my disks without waiting for sluggish UI responses.

**Why this priority**: The current performance issues affect every user interaction and create a poor user experience. Establishing persistent D-Bus connections will improve all subsequent operations.

**Independent Test**: Can be fully tested by measuring application startup time and event response times before and after implementation. The improvement should be immediately noticeable.

**Acceptance Scenarios**:

1. **Given** the storage application is not running, **When** the user launches the application, **Then** the main window appears with disk information loaded in under 3 seconds
2. **Given** the application is running, **When** a disk hotplug event occurs (USB drive inserted), **Then** the UI updates to show the new disk within 1 second
3. **Given** the application is running, **When** the user clicks on a disk to view details, **Then** the details panel updates within 500 milliseconds
4. **Given** multiple client operations are requested, **When** they execute, **Then** they reuse the same underlying connection rather than creating new connections

---

### User Story 2 - Protection Against Accidental System Unmount (Priority: P1)

As a system administrator or regular user, I want the application to prevent me from accidentally unmounting critical system partitions (like root filesystem, /boot, /home), so that I do not crash my system or lose data.

**Why this priority**: This is a safety-critical feature that prevents system instability and data loss. A user accidentally killing processes on the root filesystem could render the system unusable.

**Independent Test**: Can be fully tested by attempting to unmount a system path with kill_processes enabled and verifying that an error is returned and displayed to the user.

**Acceptance Scenarios**:

1. **Given** a filesystem is mounted at `/` (root), **When** the user attempts to unmount it with the kill_processes option, **Then** the operation is rejected with an error message explaining that system paths cannot be unmounted with process killing
2. **Given** a filesystem is mounted at `/boot`, **When** the user attempts to unmount it with the kill_processes option, **Then** the operation is rejected with an appropriate error
3. **Given** a filesystem is mounted at `/home`, **When** the user attempts to unmount it with the kill_processes option, **Then** the operation is rejected with an appropriate error
4. **Given** a filesystem is mounted at a non-system path like `/mnt/data`, **When** the user attempts to unmount it with the kill_processes option, **Then** the operation proceeds normally
5. **Given** a protected system path unmount is attempted, **When** the error is returned, **Then** the error is displayed to the user in the dialog using the existing error display helper

---

### User Story 3 - Centralized Filesystem Tool Detection (Priority: P2)

As a developer or system integrator, I want filesystem tool availability checks to be centralized in the storage service, so that the UI can query the service to determine which features to enable rather than duplicating detection logic.

**Why this priority**: This improves maintainability by consolidating detection logic in one place and enables dynamic feature enablement based on available system tools.

**Independent Test**: Can be fully tested by querying the service's supported features and verifying it accurately reflects installed system tools.

**Acceptance Scenarios**:

1. **Given** the storage service is running, **When** a client queries the supported features, **Then** the service returns an accurate list of available filesystem types based on installed tools
2. **Given** mkfs.btrfs is installed, **When** the service starts, **Then** "btrfs" is included in the supported features list
3. **Given** mkfs.xfs is NOT installed, **When** the service starts, **Then** "xfs" is NOT included in the supported features list
4. **Given** the UI needs to know which filesystem types to offer, **When** it queries the service, **Then** only supported types are shown in the format dialog
5. **Given** the service reports feature availability, **When** the main UI initializes, **Then** it can enable/disable UI elements based on service capabilities

---

### Edge Cases

- What happens when the D-Bus connection is lost during operation? The application should attempt to reconnect gracefully.
- What happens when a mount point is a subdirectory of a protected path (e.g., `/boot/efi`)? Subdirectories of protected paths should also be protected.
- What happens when the filesystem tools change (installed/uninstalled) while the service is running? The service should either re-scan on demand or provide a refresh mechanism.
- What happens when multiple protected paths exist with symlinks? The canonical path should be used for comparison.

## Requirements *(mandatory)*

### Functional Requirements

#### Performance / Persistent Connections

- **FR-001**: The UI application MUST reuse a single D-Bus system bus connection across all client operations
- **FR-002**: The connection manager MUST provide access to the shared connection for creating multiple service proxies
- **FR-003**: Client initialization MUST NOT create a new D-Bus connection for each client instance
- **FR-004**: The connection MUST be established lazily on first use and cached for subsequent operations

#### System Path Protection

- **FR-005**: The service MUST define a list of protected system mount paths (/, /boot, /home, /usr, /var, /etc, /opt, /srv)
- **FR-006**: The unmount operation with kill_processes=true MUST check if the target mount point is a protected system path
- **FR-007**: If the mount point is protected, the service MUST return an error indicating that killing processes on system paths is not permitted
- **FR-008**: The error message MUST clearly explain why the operation was rejected
- **FR-009**: The UI MUST display the error message to the user through the existing dialog error display mechanism

#### FSTools Consolidation

- **FR-010**: The service MUST provide a comprehensive filesystem tool detection mechanism that checks for all supported filesystem types
- **FR-011**: The service MUST expose the detected filesystem capabilities through the existing `supported_features` property or a new dedicated method
- **FR-012**: The UI MUST be able to query the service for filesystem tool availability instead of performing local detection
- **FR-013**: The detection MUST include tools for: ext4, xfs, btrfs, vfat, ntfs, exfat, f2fs, udf
- **FR-014**: Feature enablement in the UI MUST be driven by service-reported capabilities

### Key Entities

- **SharedConnection**: A singleton or static reference that holds the cached D-Bus system bus connection, allowing multiple proxies to share the same underlying connection
- **ProtectedPath**: A system path that requires special handling during unmount operations (e.g., /, /boot, /home)
- **FilesystemToolInfo**: Information about a filesystem type including its name, the required command (e.g., mkfs.ext4), and availability status
- **ServiceCapabilities**: The set of features the storage service supports based on installed system tools

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Application startup time is reduced by at least 50% (measured from launch to displaying disk information)
- **SC-002**: Event response time (e.g., disk hotplug detection) is under 1 second
- **SC-003**: Attempting to kill processes on a protected system path returns a clear error message without performing any destructive action
- **SC-004**: The service accurately reports all installed filesystem tools with zero false positives or negatives
- **SC-005**: UI elements for unsupported filesystem types are hidden or disabled based on service capabilities
- **SC-006**: No duplicate filesystem tool detection code exists in the UI layer

## Assumptions

- The D-Bus system bus connection remains stable during application runtime; reconnection handling can be addressed separately if needed
- Protected system paths are defined as a static list; dynamic detection of critical mounts could be a future enhancement
- Filesystem tool detection occurs at service startup; runtime changes to installed tools require a service restart
- The existing dialog error display helper is sufficient for surfacing system path protection errors

---

## APPENDIX A: Storage-DBus → UDisks2 Connection Layer

*Added during planning phase: Investigation revealed a second performance bottleneck in the storage-dbus library layer connecting to UDisks2.*

### User Story 4 - Efficient Service-to-UDisks2 Communication (Priority: P1)

As the storage service, I want to reuse a single D-Bus connection to UDisks2 across all disk discovery operations, so that service operations are fast and don't waste resources establishing redundant connections.

**Why this priority**: The `get_disks_with_volumes()` function is called 9+ times in the service layer, and each call currently creates a new D-Bus connection. This is a major source of the reported performance problems.

**Independent Test**: Can be tested by measuring the time of repeated disk enumeration operations before and after connection pooling.

**Acceptance Scenarios**:

1. **Given** the storage service is running, **When** `get_disks_with_volumes()` is called multiple times, **Then** all calls reuse the same underlying D-Bus connection to UDisks2
2. **Given** the storage-dbus library is initialized, **When** the first disk discovery is requested, **Then** a connection is established and cached
3. **Given** a cached connection exists, **When** subsequent disk operations are performed, **Then** no new D-Bus connections are created
4. **Given** DiskManager exists, **When** disk discovery is needed, **Then** it uses its cached connection rather than creating a new one

### Functional Requirements - Storage-DBus Layer

#### Performance / UDisks2 Connection Pooling

- **FR-015**: The storage-dbus library MUST maintain a single shared D-Bus system bus connection for all UDisks2 operations
- **FR-016**: The `get_disks_with_volumes()` function MUST use a cached connection instead of creating `Connection::system()` on each call
- **FR-017**: The DiskManager struct MUST expose its connection for reuse by discovery functions
- **FR-018**: Connection caching MUST be thread-safe (using Arc) to support concurrent operations
- **FR-019**: The connection MUST be established lazily on first disk discovery and reused thereafter

### Key Entities - Storage-DBus Layer

- **CachedConnection**: An Arc-wrapped zbus::Connection stored in DiskManager, shared across all disk discovery and enumeration operations
- **ConnectionHandle**: A handle type that provides access to the cached connection, ensuring proper lifetime management

### Updated Architecture Context

```
┌─────────────────────────────────────────────────────────────┐
│                     storage-ui (GUI)                        │
│  [NEW] SharedConnection → single zbus::Connection          │
└─────────────────────────┬───────────────────────────────────┘
                          │ D-Bus (system bus) - REUSED
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  storage-service (D-Bus Service)            │
└─────────────────────────┬───────────────────────────────────┘
                          │ library calls
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  storage-dbus (Library)                     │
│  [NEW] CachedConnection → single zbus::Connection          │
└─────────────────────────┬───────────────────────────────────┘
                          │ D-Bus (system bus) - REUSED
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    UDisks2 (System Service)                 │
└─────────────────────────────────────────────────────────────┘
```

### Additional Success Criteria

- **SC-007**: The `get_disks_with_volumes()` function creates zero new D-Bus connections after the first call (verified via logging/metrics)
- **SC-008**: Disk enumeration operations complete in under 500ms after initial connection is established
- **SC-009**: No duplicate connection creation code exists in storage-dbus discovery functions

---

## APPENDIX B: Polkit Authorization & User Context Passthrough

*Added during implementation: Security audit revealed that Polkit authorization checks were validating against the service (root) instead of the actual D-Bus caller, completely bypassing authorization protections.*

### Background

The storage-service uses Polkit (via zbus_polkit crate) to authorize destructive operations like formatting disks, modifying partitions, and mounting/unmounting filesystems. However, the current implementation has two critical security issues:

1. **Polkit bypass**: The `check_polkit_auth()` function uses `connection.unique_name()` which returns the service's own bus name, not the caller's. This means all authorization checks pass because root is always authorized.

2. **UDisks2 user context loss**: When the service calls UDisks2 on behalf of a user, UDisks2 sees the request coming from root. Operations like mounting create mount points owned by root (`/run/media/root/`) instead of the actual user, making them inaccessible.

### User Story 5 - Proper Polkit Authorization for All Service Methods (Priority: P1)

As a system administrator, I want all destructive storage operations to properly check Polkit authorization against the actual calling user, so that unauthorized users cannot perform dangerous operations and authorized users see appropriate password prompts.

**Why this priority**: This is a critical security vulnerability. Currently, ANY user can perform ANY operation (format disks, delete partitions, etc.) without authentication because the authorization check validates against root instead of the caller.

**Independent Test**: Can be fully tested by having an unprivileged user attempt a destructive operation and verifying:
1. A Polkit password prompt appears (if policy requires it)
2. The operation is denied if the user cancels or enters wrong credentials
3. The operation proceeds only after successful authentication

**Acceptance Scenarios**:

1. **Given** an unprivileged user runs the storage UI, **When** they attempt to format a disk, **Then** a Polkit authentication prompt appears requesting their password or admin credentials
2. **Given** the user cancels the Polkit prompt, **When** the operation is attempted, **Then** the operation is denied with an "Not authorized" error
3. **Given** the user enters incorrect credentials, **When** the operation is attempted, **Then** the operation is denied and the user can retry
4. **Given** the user provides correct credentials, **When** the operation is attempted, **Then** the operation proceeds and completes successfully
5. **Given** an admin user performs an operation, **When** the Polkit policy allows active admins without prompt, **Then** the operation proceeds immediately

---

### User Story 6 - User-Owned Mount Points and File Ownership (Priority: P1)

As a regular user, I want filesystems mounted through the storage application to be owned by me, so that I can actually access my files without needing root privileges.

**Why this priority**: Currently, mounts appear under `/run/media/root/` and files are owned by root, making them inaccessible to the user who requested the mount.

**Independent Test**: Can be fully tested by:
1. Mounting a USB drive as a non-root user
2. Verifying the mount point is under `/run/media/<username>/`
3. Verifying the user can read/write files on the mounted filesystem

**Acceptance Scenarios**:

1. **Given** a regular user mounts a USB drive, **When** the mount completes, **Then** the mount point is created under `/run/media/<username>/`
2. **Given** a regular user mounts a FAT/NTFS filesystem, **When** the mount completes, **Then** files on the filesystem are owned by that user (not root)
3. **Given** a regular user mounts a filesystem, **When** they browse the mount, **Then** they can create, read, and modify files without sudo
4. **Given** a mounted filesystem, **When** the user requests to unmount it, **Then** they can unmount it without admin credentials (if they mounted it)

---

### Functional Requirements - Polkit Authorization

#### Authorized Interface Macro

- **FR-020**: A procedural macro `#[authorized_interface()]` MUST be created that wraps the zbus `#[interface]` macro
- **FR-021**: The macro MUST automatically inject Polkit authorization checking before method execution
- **FR-022**: The macro MUST support specifying the Polkit action ID as an attribute (e.g., `#[authorized_interface(action = "org.cosmic.ext.storage-service.format")]`)
- **FR-023**: The macro MUST automatically inject a `sender: CallerInfo` parameter into the function signature
- **FR-024**: The `CallerInfo` struct MUST contain: `uid: u32`, `username: Option<String>`, `sender: String`
- **FR-025**: The macro MUST extract the caller's identity from the D-Bus message header and populate `CallerInfo`
- **FR-026**: If Polkit authorization fails, the macro MUST return a `zbus::fdo::Error::AccessDenied` without executing the method body
- **FR-027**: The macro MUST preserve all existing `#[interface]` functionality (signals, properties, etc.)

#### Authorization Checking

- **FR-028**: All destructive service methods MUST use the `#[authorized_interface()]` macro or equivalent authorization
- **FR-029**: Authorization checks MUST use the caller's unique bus name from the message header, NOT `connection.unique_name()`
- **FR-030**: The `check_polkit_auth()` function MUST be deprecated or fixed to require an explicit sender parameter
- **FR-031**: Authorization MUST support `AllowUserInteraction` flag to enable password prompts

---

### Functional Requirements - UDisks2 User Context Passthrough

#### User-Dependent Operations

- **FR-032**: The service MUST identify which UDisks2 operations have user-dependent behavior
- **FR-033**: For mount operations, the service MUST pass the caller's username via UDisks2's `as-user` option
- **FR-034**: For mount operations on FAT/NTFS/exFAT, the service MUST pass the caller's UID via the `uid` mount option
- **FR-035**: For filesystem ownership operations (take ownership), the service MUST run as the requesting user

#### UDisks2 Operations Requiring User Context

The following UDisks2 operations have material dependency on the calling user:

| Operation | User Dependency | Required Passthrough |
|-----------|-----------------|---------------------|
| Filesystem.Mount() | Mount point path, file ownership | `as-user=<username>`, `uid=<uid>` |
| Filesystem.Mount() with vfat/ntfs/exfat | File ownership in filesystem | `uid=<uid>` option |
| Filesystem.SetLabel() | May require filesystem ownership | Run as user or verify ownership |
| Encrypted.Unlock() | Cleartext device ownership | May need `as-user` equivalent |
| Partition.SetType() | Partition table ownership | Usually requires auth anyway |

---

### Key Entities - Authorization Layer

- **CallerInfo**: A struct containing the caller's identity extracted from D-Bus message header
  - `uid: u32` - Unix user ID of the calling process
  - `username: Option<String>` - Username resolved from UID (via getpwuid)
  - `sender: String` - D-Bus unique bus name of the caller (e.g., ":1.42")

- **AuthorizedInterface**: A procedural macro attribute that:
  - Wraps `#[zbus::interface]` functionality
  - Injects pre-call Polkit authorization
  - Provides caller identity to the method body

- **PolkitAction**: An attribute specifying the Polkit action ID for a method
  - Maps to actions defined in `data/polkit-1/actions/org.cosmic.ext.storage-service.policy`

---

### Technical Design - Authorized Interface Macro

```rust
// Example usage of the proposed macro
#[authorized_interface(name = "org.cosmic.ext.StorageService.Filesystems")]
impl Filesystems {
    #[authorized_interface(action = "org.cosmic.ext.storage-service.mount")]
    pub async fn mount(
        &self,
        sender: CallerInfo,  // Auto-injected by macro
        device: String,
        mount_point: String,
        options: MountOptions,
    ) -> zbus::fdo::Result<String> {
        // Method body - authorization already checked
        // sender.uid and sender.username available for UDisks2 passthrough
        crate::filesystem::mount_filesystem(&device, &mount_point, options, Some(sender.uid)).await
    }
}
```

The macro expands to:
1. Standard `#[zbus::interface]` with `#[zbus(header)]` and `#[zbus(connection)]` parameters
2. Extract caller info from header and connection
3. Perform Polkit check against the actual caller
4. If authorized, populate `CallerInfo` and call method body
5. If not authorized, return `AccessDenied` error

---

### Additional Success Criteria

- **SC-010**: All destructive service methods require proper Polkit authorization (verified by attempting operations as unprivileged user)
- **SC-011**: Polkit password prompts appear when required by policy (verified manually)
- **SC-012**: Mount points are created under `/run/media/<username>/` not `/run/media/root/` (verified by mounting as non-root user)
- **SC-013**: Files on mounted FAT/NTFS filesystems are owned by the mounting user (verified via `ls -la`)
- **SC-014**: The `check_polkit_auth()` function is either removed or requires explicit sender parameter
- **SC-015**: No authorization checks use `connection.unique_name()` to identify the caller
