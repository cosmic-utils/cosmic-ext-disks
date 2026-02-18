# D-Bus API Contract: RClone Interface

**Feature**: 072-rclone-mounts
**Interface**: `org.cosmic.ext.StorageService.Rclone`
**Path**: `/org/cosmic/ext/StorageService/rclone`
**Date**: 2026-02-17

## Interface Definition

```
org.cosmic.ext.StorageService.Rclone
```

This interface provides RClone mount management operations with polkit-based authorization.

## Methods

### list_remotes

List all configured RClone remotes from both user and system config files.

**Authorization**: `org.cosmic.ext.storage-service.rclone-read` (no auth)

**Signature**:
```dbus
list_remotes() -> String (JSON)
```

**Returns**: JSON-encoded `RemoteConfigList`

```json
{
  "remotes": [
    {
      "name": "my-drive",
      "remote_type": "drive",
      "scope": "User",
      "options": { "type": "drive" },
      "has_secrets": true
    }
  ],
  "user_config_path": "/home/user/.config/rclone/rclone.conf",
  "system_config_path": "/etc/rclone.conf"
}
```

**Errors**:
- `org.freedesktop.DBus.Error.Failed`: RClone not installed or config read error

---

### get_remote

Get detailed configuration for a specific remote.

**Authorization**: `org.cosmic.ext.storage-service.rclone-read` (no auth)

**Signature**:
```dbus
get_remote(name: String, scope: String) -> String (JSON)
```

**Parameters**:
- `name`: Remote name
- `scope`: "user" or "system"

**Returns**: JSON-encoded `RemoteConfig`

**Errors**:
- `org.freedesktop.DBus.Error.Failed`: Remote not found

---

### test_remote

Test connectivity and authentication for a remote.

**Authorization**: `org.cosmic.ext.storage-service.rclone-test` (no auth)

**Signature**:
```dbus
test_remote(name: String, scope: String) -> String (JSON)
```

**Parameters**:
- `name`: Remote name
- `scope`: "user" or "system"

**Returns**: JSON-encoded test result

```json
{
  "success": true,
  "message": "Connection successful",
  "latency_ms": 234
}
```

Or on failure:
```json
{
  "success": false,
  "message": "Authentication failed: invalid token",
  "latency_ms": 1500
}
```

**Errors**:
- `org.freedesktop.DBus.Error.Failed`: Test execution error

---

### mount

Start an RClone mount for a remote.

**Authorization**:
- User scope: No authorization required
- System scope: `org.cosmic.ext.storage-service.rclone-mount` (auth_admin_keep)

**Signature**:
```dbus
mount(name: String, scope: String) -> ()
```

**Parameters**:
- `name`: Remote name
- `scope`: "user" or "system"

**Returns**: Nothing on success

**Errors**:
- `org.freedesktop.DBus.Error.Failed`: Mount failed (already mounted, config error, etc.)
- `org.freedesktop.DBus.Error.AccessDenied`: Authorization denied (system scope)

---

### unmount

Stop an RClone mount.

**Authorization**:
- User scope: No authorization required
- System scope: `org.cosmic.ext.storage-service.rclone-mount` (auth_admin_keep)

**Signature**:
```dbus
unmount(name: String, scope: String) -> ()
```

**Parameters**:
- `name`: Remote name
- `scope`: "user" or "system"

**Returns**: Nothing on success

**Errors**:
- `org.freedesktop.DBus.Error.Failed`: Unmount failed (not mounted, busy, etc.)
- `org.freedesktop.DBus.Error.AccessDenied`: Authorization denied (system scope)

---

### get_mount_status

Get current mount status for a remote.

**Authorization**: `org.cosmic.ext.storage-service.rclone-read` (no auth)

**Signature**:
```dbus
get_mount_status(name: String, scope: String) -> String (JSON)
```

**Parameters**:
- `name`: Remote name
- `scope`: "user" or "system"

**Returns**: JSON-encoded status

```json
{
  "status": "Mounted",
  "mount_point": "/home/user/mnt/my-drive"
}
```

---

### create_remote

Create a new RClone remote configuration.

**Authorization**: `org.cosmic.ext.storage-service.rclone-config` (auth_admin_keep for system scope)

**Signature**:
```dbus
create_remote(config: String (JSON), scope: String) -> ()
```

**Parameters**:
- `config`: JSON-encoded `RemoteConfig` (without secrets populated)
- `scope`: "user" or "system"

**Returns**: Nothing on success

**Errors**:
- `org.freedesktop.DBus.Error.Failed`: Remote already exists, write error
- `org.freedesktop.DBus.Error.AccessDenied`: Authorization denied (system scope)

---

### update_remote

Update an existing RClone remote configuration.

**Authorization**: `org.cosmic.ext.storage-service.rclone-config` (auth_admin_keep for system scope)

**Signature**:
```dbus
update_remote(name: String, config: String (JSON), scope: String) -> ()
```

**Parameters**:
- `name`: Existing remote name
- `config`: JSON-encoded `RemoteConfig` with updated options
- `scope`: "user" or "system"

**Returns**: Nothing on success

**Errors**:
- `org.freedesktop.DBus.Error.Failed`: Remote not found, write error
- `org.freedesktop.DBus.Error.AccessDenied`: Authorization denied (system scope)

---

### delete_remote

Delete an RClone remote configuration.

**Authorization**: `org.cosmic.ext.storage-service.rclone-config` (auth_admin_keep for system scope)

**Signature**:
```dbus
delete_remote(name: String, scope: String) -> ()
```

**Parameters**:
- `name`: Remote name to delete
- `scope`: "user" or "system"

**Returns**: Nothing on success

**Errors**:
- `org.freedesktop.DBus.Error.Failed`: Remote not found, write error
- `org.freedesktop.DBus.Error.AccessDenied`: Authorization denied (system scope)

## Signals

### mount_changed

Emitted when a mount status changes.

**Signature**:
```dbus
mount_changed(name: String, scope: String, status: String) -> ()
```

**Parameters**:
- `name`: Remote name
- `scope`: "user" or "system"
- `status`: New status ("Mounted", "Unmounted", "Error")

## Properties

### supported_remote_types

List of supported RClone remote backend types.

**Type**: `Array<String>`
**Access**: Read

```dbus
supported_remote_types: ["drive", "s3", "dropbox", "onedrive", "ftp", "sftp", "webdav"]
```

## Polkit Actions Summary

| Action ID | Auth Level | Methods |
|-----------|------------|---------|
| `org.cosmic.ext.storage-service.rclone-read` | `yes` | list_remotes, get_remote, get_mount_status |
| `org.cosmic.ext.storage-service.rclone-test` | `yes` | test_remote |
| `org.cosmic.ext.storage-service.rclone-mount` | `auth_admin_keep` | mount, unmount (system scope only) |
| `org.cosmic.ext.storage-service.rclone-config` | `auth_admin_keep` | create_remote, update_remote, delete_remote (system scope only) |
