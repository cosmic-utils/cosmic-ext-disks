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
