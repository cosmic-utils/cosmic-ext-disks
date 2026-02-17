# Feature Specification: RClone Mount Management

**Feature Branch**: `072-rclone-mounts`
**Created**: 2026-02-17
**Status**: Draft
**GitHub Issue**: https://github.com/cosmic-utils/cosmic-ext-storage/issues/72
**Input**: User description: "We are to add support for managing RClone mounts. We should support read/write of rclone.conf files from all rclone.conf expected locations. We should support Start/Stop/Restart the Rclone daemon. We should not run our own instance of the daemon. This should live in storage-sys. Rclone mounts are to appear under a Network section on the Sidebar. We will be supporting Samba and FTP in the future, so generalise supporting frameworks as it makes sense. If there is a test functionality in rclone CLI to validate config, we should expose that as an action too."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Network Mounts (Priority: P1)

As a user, I want to see all my configured RClone mounts in a dedicated "Network" section of the sidebar so that I can easily discover and access my cloud storage mount points alongside local storage.

**Why this priority**: Users need visibility of their network mounts before they can manage them. This is the foundation for all other functionality.

**Independent Test**: Can be fully tested by configuring an RClone remote externally, opening the application, and verifying the mount appears under the Network section in the sidebar.

**Acceptance Scenarios**:

1. **Given** the user has RClone remotes configured, **When** they open the storage application, **Then** a "Network" section is visible in the sidebar containing all configured RClone remotes.
2. **Given** the user has no RClone remotes configured, **When** they open the storage application, **Then** the "Network" section shows an empty state or helpful message.
3. **Given** the user has multiple RClone remotes configured, **When** they view the Network section, **Then** each remote is displayed with its name, current status (mounted/unmounted), and scope indicator (user/system).

---

### User Story 2 - Control Mount Daemon (Priority: P2)

As a user, I want to start, stop, and restart individual RClone mounts so that I can control which cloud storage is accessible without managing system services directly.

**Why this priority**: Once users can see their mounts, controlling them is the next critical action. This enables actual use of the mounts.

**Independent Test**: Can be tested by selecting a configured remote and verifying start/stop/restart actions correctly change the mount state.

**Acceptance Scenarios**:

1. **Given** an RClone remote is not mounted, **When** the user selects "Start", **Then** the mount is activated and becomes accessible in the file system.
2. **Given** an RClone remote is mounted, **When** the user selects "Stop", **Then** the mount is deactivated and removed from the file system.
3. **Given** an RClone remote is mounted, **When** the user selects "Restart", **Then** the mount is stopped and then started again.
4. **Given** an RClone remote is in an error state, **When** the user selects "Restart", **Then** the mount attempts to recover by stopping and starting.

---

### User Story 3 - Test Remote Configuration (Priority: P3)

As a user, I want to validate my RClone remote configuration before attempting to mount so that I can identify and fix configuration issues proactively.

**Why this priority**: Testing configuration helps users avoid errors, but it is not essential for basic mount management.

**Independent Test**: Can be tested by selecting a remote and invoking the test action, then verifying the result matches the actual configuration validity.

**Acceptance Scenarios**:

1. **Given** an RClone remote has valid configuration, **When** the user selects "Test Configuration", **Then** a success message is displayed confirming the configuration works.
2. **Given** an RClone remote has invalid configuration (wrong credentials, inaccessible endpoint), **When** the user selects "Test Configuration", **Then** an error message is displayed explaining the issue.
3. **Given** the user is testing a configuration, **When** the test is in progress, **Then** a loading indicator is shown until completion.

---

### User Story 4 - Manage Remote Configuration (Priority: P4)

As a user, I want to view and edit RClone remote configurations through the application so that I can set up cloud storage connections without manually editing configuration files.

**Why this priority**: Configuration management is valuable for convenience but users can always configure RClone externally if needed.

**Independent Test**: Can be tested by creating a new remote configuration through the interface and verifying it appears both in the application and in the rclone.conf file.

**Acceptance Scenarios**:

1. **Given** the user wants to add a new remote, **When** they provide the required configuration details and save, **Then** the remote is added to rclone.conf and appears in the Network section.
2. **Given** the user wants to modify an existing remote, **When** they update configuration details and save, **Then** the rclone.conf file is updated with the new settings.
3. **Given** the user wants to remove a remote, **When** they confirm deletion, **Then** the remote is removed from rclone.conf and the Network section.
4. **Given** multiple rclone.conf file locations exist, **When** the application reads configuration, **Then** all standard rclone.conf locations are checked and configurations are merged for display.

---

### Edge Cases

- What happens when the RClone daemon service is not running on the system?
- How does the system handle concurrent mount/unmount requests for the same remote?
- What happens when rclone.conf contains malformed configuration?
- How does the system handle remotes that require interactive authentication?
- What happens when the mount point directory does not exist or is not accessible?
- How does the system behave when network connectivity is lost during a mount operation?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display all configured RClone remotes in a "Network" section of the sidebar.
- **FR-002**: System MUST read RClone configuration from all standard rclone.conf file locations (per-user primary, system-wide as fallback).
- **FR-003**: System MUST write RClone configuration changes to the appropriate rclone.conf location based on scope.
- **FR-004**: System MUST allow users to start an RClone mount for a configured remote (per-user mounts without elevation, system-wide mounts require polkit authorization).
- **FR-005**: System MUST allow users to stop an active RClone mount (per-user without elevation, system-wide requires polkit).
- **FR-006**: System MUST allow users to restart an RClone mount (per-user without elevation, system-wide requires polkit).
- **FR-007**: System MUST NOT spawn its own RClone daemon instance; it must interact with the existing system service.
- **FR-008**: System MUST display the current status (mounted/unmounted/error) of each remote.
- **FR-009**: System MUST provide a configuration test action that validates remote connectivity and credentials.
- **FR-010**: System MUST support creating new RClone remote configurations.
- **FR-011**: System MUST support editing existing RClone remote configurations.
- **FR-012**: System MUST support deleting RClone remote configurations.
- **FR-013**: System MUST use a generalized network mount framework to support future addition of Samba and FTP mount types.
- **FR-014**: System MUST provide meaningful error messages when mount operations fail.
- **FR-015**: System MUST respect file permissions when reading and writing rclone.conf files.
- **FR-016**: System MUST define four polkit actions for system-wide RClone operations with auth levels: `rclone-read` (no auth), `rclone-test` (no auth), `rclone-mount` (auth_admin_keep), `rclone-config` (auth_admin_keep).
- **FR-017**: System MUST display a visual indicator (badge or icon) for each remote showing whether it is user-scoped or system-scoped.

### Key Entities

- **NetworkMount**: Represents a mountable network storage resource. Contains name, mount type (RClone, future: Samba, FTP), current status, and mount point path (per-user: `~/mnt/<remote-name>/`, system-wide: `/mnt/rclone/<remote-name>/`). Abstract to support multiple backend types.

- **RemoteConfiguration**: Represents the configuration for a network storage provider. Contains provider type, credentials (secured), connection settings, and mount options. Type-specific for RClone but extensible for other protocols.

- **MountStatus**: Represents the operational state of a network mount. States include: Unmounted, Mounting, Mounted, Unmounting, Error. Includes optional error details when in error state.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can view all configured RClone remotes within 2 seconds of opening the Network section.
- **SC-002**: Mount and unmount operations complete within 5 seconds for responsive cloud providers.
- **SC-003**: Configuration test operations complete within 10 seconds or provide progress feedback.
- **SC-004**: 95% of mount operations succeed when configuration is valid and network is available.
- **SC-005**: Users receive clear error messages for 100% of failed operations, indicating the cause and suggested remediation.
- **SC-006**: The generalized framework supports adding a new mount type (e.g., Samba) with no changes to the UI layer's mount-agnostic components.

## Clarifications

### Session 2026-02-17

- Q: RClone scope (per-user vs system-wide)? → A: Both - Support per-user configs primarily (`~/.config/rclone/rclone.conf`), allow system-wide (`/etc/rclone.conf`) as fallback. Polkit elevation required for system-wide mount operations.
- Q: Polkit action structure? → A: Four separate actions: `rclone-read` (view configs), `rclone-mount` (start/stop/restart), `rclone-config` (create/edit/delete remotes), `rclone-test` (validate configuration) - each with separate auth levels.
- Q: Polkit auth levels for system-wide operations? → A: `rclone-read`: no auth, `rclone-test`: no auth, `rclone-mount`: auth_admin_keep, `rclone-config`: auth_admin_keep.
- Q: Default mount point location? → A: Per-user mounts: `~/mnt/<remote-name>/`, System-wide mounts: `/mnt/rclone/<remote-name>/`.
- Q: Distinguish config scope in UI? → A: Yes - Display a badge or icon indicating whether each remote is user-scoped or system-scoped.

## Assumptions

- RClone is installed and available on the target system.
- RClone configuration follows standard file locations (per-user: `~/.config/rclone/rclone.conf`, system-wide: `/etc/rclone.conf`).
- Per-user configurations are the primary scope; system-wide configurations are supported with polkit elevation.
- Systemd user units manage per-user RClone mounts; system units manage system-wide mounts.
- Users have appropriate file system permissions to read/write their own rclone.conf files.
- Network connectivity is required for mount and test operations to succeed.
- The storage-service provides this capability via D-Bus with polkit authorization.

## Out of Scope

- Creating or managing the system service for RClone daemon.
- Providing interactive authentication flows for remotes that require browser-based OAuth.
- Mounting remotes on behalf of other users.
- Advanced RClone features like caching, filtering, or bandwidth limiting configuration through the UI.
- Real-time synchronization status monitoring.
