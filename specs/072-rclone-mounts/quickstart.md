# Quickstart: RClone Mount Management

**Feature**: 072-rclone-mounts
**Date**: 2026-02-17

## Prerequisites

1. **RClone installed**: `sudo apt install rclone` or download from https://rclone.org/install/
2. **COSMIC Ext Storage Service**: Built and installed from this repository
3. **Polkit policy**: Installed to `/usr/share/polkit-1/actions/`

## Quick Test

### 1. Configure a test remote

```bash
# Interactive configuration (recommended for first time)
rclone config

# Or create a minimal test config
mkdir -p ~/.config/rclone
cat > ~/.config/rclone/rclone.conf << EOF
[test-local]
type = alias
remote = /tmp
EOF
```

### 2. Test via D-Bus

```bash
# List remotes
gdbus call --system \
  --dest org.cosmic.ext.StorageService \
  --object-path /org/cosmic/ext/StorageService/rclone \
  --method org.cosmic.ext.StorageService.Rclone.list_remotes

# Test a remote
gdbus call --system \
  --dest org.cosmic.ext.StorageService \
  --object-path /org/cosmic/ext/StorageService/rclone \
  --method org.cosmic.ext.StorageService.Rclone.test_remote "test-local" "user"

# Mount a remote
gdbus call --system \
  --dest org.cosmic.ext.StorageService \
  --object-path /org/cosmic/ext/StorageService/rclone \
  --method org.cosmic.ext.StorageService.Rclone.mount "test-local" "user"

# Check mount status
ls ~/mnt/test-local/

# Unmount
gdbus call --system \
  --dest org.cosmic.ext.StorageService \
  --object-path /org/cosmic/ext/StorageService/rclone \
  --method org.cosmic.ext.StorageService.Rclone.unmount "test-local" "user"
```

### 3. Verify polkit actions

```bash
# Check polkit can authorize system-wide operations
pkaction --action-id org.cosmic.ext.storage-service.rclone-mount --verbose
```

## Development Setup

### Build

```bash
# Build all crates
cargo build --workspace

# Build just the service
cargo build -p storage-service
```

### Run service (development)

```bash
# Run with debug logging
RUST_LOG=storage_service=debug sudo -E cargo run -p storage-service
```

### Test

```bash
# Run tests
cargo test -p storage-service -- rclone

# Run with verbose output
cargo test -p storage-service -- rclone --nocapture
```

## File Structure After Implementation

```
storage-common/src/rclone.rs      # Data models
storage-sys/src/rclone.rs         # CLI operations
storage-service/src/rclone.rs     # D-Bus interface
data/polkit-1/actions/*.policy    # Polkit actions (updated)
```

## Common Operations

### Adding a new remote type

1. Add to `supported_remote_types` in `RcloneHandler`
2. Add validation rules to `RemoteConfig` if needed
3. No changes to mount/unmount logic required

### Debugging mount issues

```bash
# Check if rclone is installed
which rclone
rclone version

# Check mount status manually
mount | grep rclone
mountpoint ~/mnt/test-local

# Check service logs
journalctl -u cosmic-storage-service -f
```

## UI Integration Points

The UI should:

1. **Call `list_remotes` on startup** to populate the Network section
2. **Subscribe to `mount_changed` signal** for real-time updates
3. **Display scope badge** (user/system) per FR-017
4. **Show mount status icon** (mounted/unmounted/error)
5. **Prompt for polkit auth** when performing system-scope operations
