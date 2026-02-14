# Quick Reference: Converting TODOs to Real Code

This guide shows how to convert the TODO comments into actual working code.

---

## Client Initialization

### Add to AppModel

```rust
// storage-ui/src/ui/app/state.rs

use crate::client::*;
use std::sync::Arc;

pub struct AppModel {
    // ... existing fields ...
    
    // Storage service clients
    pub disks_client: Arc<DisksClient>,
    pub partitions_client: Arc<PartitionsClient>,
    pub filesystems_client: Arc<FilesystemsClient>,
    pub luks_client: Arc<LuksClient>,
    pub btrfs_client: Arc<BtrfsClient>,
    pub image_client: Arc<ImageClient>,
}
```

### Initialize on Startup

```rust
// In app initialization
impl Application for AppModel {
    async fn new() -> (Self, Task<Self::Message>) {
        let disks_client = Arc::new(DisksClient::new().await.expect("DisksClient"));
        let partitions_client = Arc::new(PartitionsClient::new().await.expect("PartitionsClient"));
        let filesystems_client = Arc::new(FilesystemsClient::new().await.expect("FilesystemsClient"));
        let luks_client = Arc::new(LuksClient::new().await.expect("LuksClient"));
        let btrfs_client = Arc::new(BtrfsClient::new().await.expect("BtrfsClient"));
        let image_client = Arc::new(ImageClient::new().await.expect("ImageClient"));
        
        let app = Self {
            disks_client,
            partitions_client,
            filesystems_client,
            luks_client,
            btrfs_client,
            image_client,
            // ... other fields ...
        };
        
        (app, Task::none())
    }
}
```

---

## Btrfs Operations (Already Done!)

These are already implemented, but here's the pattern:

```rust
// OLD
let btrfs = BtrfsFilesystem::new(mount_path).await?;
let subvolumes = btrfs.list_subvolumes().await?;

// NEW
let btrfs_client = BtrfsClient::new().await?;
let subvol_list = btrfs_client.list_subvolumes(&mount_point).await?;
let subvolumes = subvol_list.subvolumes;
```

---

## Mount Operations

### Find and Replace Pattern

```rust
// OLD
volume.mount().await?

// NEW (when you have access to app)
let device = volume.device_path.as_ref().unwrap();
app.filesystems_client.mount(device, "", None).await?

// OR (in Task::perform)
let filesystems_client = FilesystemsClient::new().await?;
filesystems_client.mount(&device, "", None).await?
```

### Example: volumes/update/mount.rs

**BEFORE:**
```rust
pub(super) fn mount(control: &mut VolumesControl) -> Task<cosmic::Action<Message>> {
    let Some(volume) = control.segments.get(control.selected_segment)
        .and_then(|s| s.volume.clone())
    else {
        return Task::none();
    };

    let object_path = volume.path.to_string();
    // TODO: Replace with FilesystemsClient
    perform_volume_operation(
        || async move { volume.mount().await },
        "mount",
        Some(object_path),
    )
}
```

**AFTER:**
```rust
pub(super) fn mount(control: &mut VolumesControl) -> Task<cosmic::Action<Message>> {
    let Some(volume) = control.segments.get(control.selected_segment)
        .and_then(|s| s.volume.clone())
    else {
        return Task::none();
    };

    let device = match volume.device_path.as_ref() {
        Some(d) => d.clone(),
        None => return Task::none(),
    };

    Task::perform(
        async move {
            let filesystems_client = FilesystemsClient::new().await?;
            filesystems_client.mount(&device, "", None).await?;
            DriveModel::get_drives().await // TODO: Replace in Phase 5
        },
        |result| match result {
            Ok(drives) => Message::UpdateNav(drives, None).into(),
            Err(e) => {
                let ctx = UiErrorContext::new("mount");
                log_error_and_show_dialog(fl!("mount-failed"), e, ctx).into()
            }
        },
    )
}
```

---

## Unmount Operations

### With Process Killing

```rust
// OLD
match volume.unmount().await {
    Ok(()) => { /* success */ }
    Err(e) => {
        if let Some(disk_err) = e.downcast_ref::<DiskError>()
            && matches!(disk_err, DiskError::ResourceBusy { .. })
        {
            // Handle busy
        }
    }
}

// NEW
let filesystems_client = FilesystemsClient::new().await?;
match filesystems_client.unmount(&device, false, false).await {
    Ok(UnmountResult::Success) => {
        // Success - reload drives
        DriveModel::get_drives().await
    }
    Ok(UnmountResult::Busy { processes, mount_point, .. }) => {
        // Show process kill dialog
        UnmountResult::Busy { device, mount_point, processes, object_path }
    }
    Err(e) => {
        // Generic error
        UnmountResult::GenericError
    }
}
```

---

## Partition Operations

### Delete

```rust
// OLD
p.delete().await?;

// NEW
let partitions_client = PartitionsClient::new().await?;
let device = p.device_path.as_ref().unwrap();
partitions_client.delete_partition(device).await?;
```

### Resize

```rust
// OLD
volume.resize(new_size).await?;

// NEW
let partitions_client = PartitionsClient::new().await?;
let device = volume.device_path.as_ref().unwrap();
partitions_client.resize_partition(device, new_size).await?;
```

### Create

```rust
// OLD
model.create_partition(create_partition_info).await?;

// NEW
let partitions_client = PartitionsClient::new().await?;
partitions_client.create_partition(
    &model.block_device, // disk device path
    create_partition_info.offset,
    create_partition_info.size,
    &create_partition_info.partition_type_id,
).await?;
```

---

## LUKS Operations

### Unlock

```rust
// OLD
p.unlock(&passphrase).await?;

// NEW
let luks_client = LuksClient::new().await?;
let device = p.device_path.as_ref().unwrap();
luks_client.unlock(device, &passphrase).await?;
```

### Lock

```rust
// OLD
p.lock().await?;

// NEW
let luks_client = LuksClient::new().await?;
let cleartext_device = p.device_path.as_ref().unwrap();
luks_client.lock(cleartext_device).await?;
```

---

## Format Operations

```rust
// OLD
volume.format(name, erase, fs_type).await?;

// NEW
let filesystems_client = FilesystemsClient::new().await?;
let device = volume.device_path.as_ref().unwrap();
let options = if erase {
    Some("{\"erase\": true}")
} else {
    None
};
filesystems_client.format(device, &fs_type, &name, options).await?;
```

---

## DriveModel::get_drives() Replacement

### Discovery Call

```rust
// OLD
let drives = DriveModel::get_drives().await?;

// NEW (once clients are in AppModel)
let disks = app.disks_client.list_disks().await?;
```

### Property Access Changes

```rust
// OLD
drive.name          → disk.label
drive.block_path    → disk.device
drive.size          → disk.size_bytes
drive.is_removable  → disk.removable
drive.serial        → disk.serial_number

drive.volumes       → Query separately
drive.volumes_flat  → Query separately
```

### Volume Querying (Separate)

```rust
// Volumes are no longer nested - query when needed
let volumes = disks_client.get_volumes(&disk.device).await?;

// Or use a volumes client if that's how it's structured
let volumes_client = VolumesClient::new().await?;
let volumes = volumes_client.list_for_disk(&disk.device).await?;
```

---

## Error Handling Migration

### DiskError → ClientError

```rust
// OLD
use disks_dbus::DiskError;

// NEW
use crate::client::ClientError;
```

### ResourceBusy Detection

```rust
// OLD
if let Some(disk_err) = e.downcast_ref::<DiskError>()
    && matches!(disk_err, DiskError::ResourceBusy { .. })
{
    // Handle busy
}

// NEW - Use UnmountResult instead
match filesystems_client.unmount(...).await {
    Ok(UnmountResult::Busy { processes, .. }) => {
        // Directly get process list from result
    }
    _ => {}
}
```

---

## Common Patterns

### Pattern 1: Simple Operation

```rust
Task::perform(
    async move {
        let client = OperationClient::new().await?;
        client.operation(&device, params).await?;
        DriveModel::get_drives().await // TODO: Replace in Phase 5
    },
    |result| match result {
        Ok(drives) => Message::UpdateNav(drives, None).into(),
        Err(e) => handle_error(e).into(),
    },
)
```

### Pattern 2: Multi-Step Operation

```rust
Task::perform(
    async move {
        let client1 = Client1::new().await?;
        let client2 = Client2::new().await?;
        
        client1.step1().await?;
        client2.step2().await?;
        
        DriveModel::get_drives().await
    },
    |result| match result {
        Ok(drives) => Message::UpdateNav(drives, None).into(),
        Err(e) => handle_error(e).into(),
    },
)
```

### Pattern 3: With Error Handling

```rust
Task::perform(
    async move {
        let client = OperationClient::new().await?;
        match client.operation(&device).await {
            Ok(result) => Ok((drives, result)),
            Err(ClientError::Busy { info }) => Err(BusyError(info)),
            Err(e) => Err(GenericError(e)),
        }
    },
    |result| match result {
        Ok((drives, _)) => Message::UpdateNav(drives, None).into(),
        Err(BusyError(info)) => Message::ShowBusyDialog(info).into(),
        Err(GenericError(e)) => handle_error(e).into(),
    },
)
```

---

## Testing Checklist

After converting each TODO:

- [ ] Code compiles
- [ ] Operation works with real disk
- [ ] Error conditions handled
- [ ] Polkit authentication works
- [ ] Busy detection works (for unmount)
- [ ] UI updates correctly
- [ ] No data loss

---

## Common Mistakes to Avoid

1. **Don't create client in hot loop** - Create once, reuse
2. **Don't forget error context** - Use UiErrorContext for logging
3. **Don't ignore client connection failures** - Handle gracefully
4. **Don't assume device_path exists** - Always check Option
5. **Don't mix old and new APIs** - Complete one operation at a time

---

## When You Get Stuck

1. Check existing BtrfsClient implementations - they're complete
2. Look at client wrapper code in storage-ui/src/client/
3. Check storage-service D-Bus interface definitions
4. Test with `busctl` to verify service responds correctly
5. Check service logs: `journalctl -fu cosmic-ext-storage-service`

---

**Good luck with the implementation!**
