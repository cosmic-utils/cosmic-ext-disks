# Research: RClone Mount Management

**Feature**: 072-rclone-mounts
**Date**: 2026-02-17

## RClone CLI Research

### Config File Locations

| Location | Scope | Priority |
|----------|-------|----------|
| `~/.config/rclone/rclone.conf` | Per-user | Primary |
| `~/.rclone.conf` | Per-user (legacy) | Secondary |
| `/etc/rclone.conf` | System-wide | Fallback |

**Decision**: Read from all locations, prefer per-user, support system-wide with polkit elevation.

**Rationale**: Matches rclone's standard behavior and allows both personal and admin-managed configurations.

### Config File Format

rclone.conf uses INI format:
```ini
[remote_name]
type = google_drive
client_id = xxx
client_secret = yyy
token = {"access_token":"xxx"}
```

**Decision**: Use Rust `ini` crate or custom parser for read/write.

**Rationale**: INI format is simple; need to preserve comments and formatting where possible.

### Listing Remotes

**Command**: `rclone listremotes`
**Output**: Remote names in `[name]` format

**Decision**: Parse output of `rclone listremotes` for discovery.

**Rationale**: Using rclone's own command ensures compatibility with all config formats.

### Testing Configuration

**Command**: `rclone ls remote:/ --max-depth 1`
**Exit Codes**:
- 0: Success
- 1-4: Various errors (network, auth, config)

**Decision**: Use `rclone ls` with `--max-depth 1` for quick validation.

**Rationale**: Tests both config validity and network connectivity. Limit depth for speed.

### Mount Operations

**Command**: `rclone mount remote: /path --daemon`
**Systemd Pattern**: User units for per-user, system units for system-wide

**Decision**: Invoke rclone mount command directly; detect mount status via `mountpoint -q`.

**Rationale**: Per spec, we do NOT run our own daemon. We invoke rclone's built-in daemon mode.

### Mount Status Detection

**Methods**:
1. `mountpoint -q /path/to/mount` - Check if path is a mount point
2. Check `/proc/mounts` for rclone entries
3. `systemctl is-active rclone-mount@remote` (if using systemd units)

**Decision**: Use `mountpoint -q` for reliable cross-distro detection.

**Rationale**: Simple, reliable, no dependency on specific init system.

## Architecture Decisions

### D-Bus Interface Pattern

**Decision**: Follow existing `BtrfsHandler` pattern with `#[authorized_interface]` macro.

**Rationale**: Consistent with existing codebase (see `storage-service/src/btrfs.rs:20-30`).

### Polkit Action Structure

| Action | Auth Level | Use Case |
|--------|------------|----------|
| `rclone-read` | `yes` (no auth) | List/view remotes |
| `rclone-test` | `yes` (no auth) | Validate config |
| `rclone-mount` | `auth_admin_keep` | Start/stop/restart mounts |
| `rclone-config` | `auth_admin_keep` | Create/edit/delete remotes |

**Decision**: Four separate actions per clarification session.

**Rationale**: Fine-grained control; read/test don't need elevation.

### Scope Handling

**Decision**: Pass scope parameter ("user" or "system") to D-Bus methods.

**Rationale**:
- Determines which config file to read/write
- Determines which mount point prefix to use
- Polkit only checks elevation for system scope

## Alternatives Considered

### Alternative: Direct config file parsing
- **Rejected**: rclone's config format may change; using rclone CLI ensures compatibility
- **Hybrid approach**: Use CLI for listing, direct file access for editing (to preserve formatting)

### Alternative: Run rclone as subprocess
- **Accepted**: We must invoke rclone CLI for mount/test operations anyway
- **Pattern**: Use `tokio::process::Command` for async execution

### Alternative: Zbus mount monitoring
- **Rejected**: Overkill for our needs; `mountpoint -q` is simpler
- **May revisit**: If real-time mount event notifications are needed

## References

- RClone documentation: https://rclone.org/docs/
- Existing patterns: `storage-service/src/btrfs.rs`
- Polkit integration: `storage-service-macros/src/lib.rs`
